use graphql_client::{GraphQLQuery, Response};
use poise::serenity_prelude as serenity;

use super::{anilist_query, clean_html, send_deletable};

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "schema.json",
	query_path = "queries/SearchStaff.graphql"
)]
struct SearchStaff;

/// Searches for a staff member by name
#[poise::command(slash_command, rename = "staff")]
pub(crate) async fn find_staff(
	ctx: crate::Context<'_>,
	#[description = "Name to search"] name: String,
) -> Result<(), crate::Error> {
	let request_body = SearchStaff::build_query(search_staff::Variables { search: name });
	let response_body: Response<search_staff::ResponseData> =
		anilist_query(&request_body).await?.json().await?;
	let staff = match response_body.data {
		Some(data) => match data.staff {
			Some(staff) => staff,
			None => {
				ctx.say("Not found").await?;
				return Ok(());
			}
		},
		None => {
			ctx.say("Not found").await?;
			return Ok(());
		}
	};

	let bio = match &staff.description {
		Some(bio) => {
			let mut bio = clean_html(bio);
			// Keep bios short, but if it has <100 characters left, just let it finish
			if bio.len() > 400 && bio.len() >= 500 {
				bio = format!("{}...", bio.split_at(400).0);
			}
			bio
		}
		None => "".to_string(),
	};

	let staff_url = &staff.site_url.unwrap();
	let embed = serenity::CreateEmbed::new()
		.author(
			serenity::CreateEmbedAuthor::new(&staff.name.unwrap().full.unwrap())
				.icon_url("https://anilist.co/img/icons/favicon-32x32.png")
				.url(staff_url),
		)
		.thumbnail(&staff.image.unwrap().large.unwrap())
		.description(bio)
		.color(3447003);

	send_deletable(
		ctx,
		poise::CreateReply {
			content: Some(format!("<{}>", &staff_url)),
			embeds: vec![embed],
			reply: true,
			..Default::default()
		},
	)
	.await?;

	Ok(())
}
