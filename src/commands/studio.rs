use graphql_client::{GraphQLQuery, Response};
use poise::serenity_prelude as serenity;

use super::{anilist_query, send_deletable};

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "schema.json",
	query_path = "queries/SearchStudio.graphql"
)]
struct SearchStudio;

/// Searches for a studio by name
#[poise::command(slash_command, rename = "studio")]
pub(crate) async fn find_studio(
	ctx: crate::Context<'_>,
	#[description = "Name to search"] name: String,
) -> Result<(), crate::Error> {
	let request_body = SearchStudio::build_query(search_studio::Variables { search: name });
	let response_body: Response<search_studio::ResponseData> =
		anilist_query(&request_body).await?.json().await?;
	let studio = match response_body.data {
		Some(data) => match data.studio {
			Some(studio) => studio,
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

	let popular_titles = collect_titles(&studio.popular);
	let latest_titles = collect_titles(&studio.latest);

	let studio_url = &studio.site_url.unwrap();
	let embed = serenity::CreateEmbed::new()
		.author(
			serenity::CreateEmbedAuthor::new(&studio.name)
				.icon_url("https://anilist.co/img/icons/favicon-32x32.png")
				.url(studio_url),
		)
		.field("Popular Titles", popular_titles, true)
		.field("Latest Titles", latest_titles, true)
		.color(3447003);

	send_deletable(
		ctx,
		poise::CreateReply {
			content: Some(format!("<{}>", studio_url)),
			embeds: vec![embed],
			reply: true,
			..Default::default()
		},
	)
	.await?;
	Ok(())
}

fn collect_titles(media: &Option<search_studio::studioMedia>) -> String {
	match media {
		Some(media) => {
			let mut titles = media
				.nodes
				.as_ref()
				.unwrap()
				.iter()
				.map(|n| n.as_ref().unwrap())
				.map(|m| {
					format!(
						"[{}]({})",
						m.title.as_ref().unwrap().romaji.as_ref().unwrap(),
						m.site_url.as_ref().unwrap()
					)
				})
				.collect::<Vec<String>>()
				.join("\n- ");

			if titles.len() > 0 {
				titles = format!("- {}", titles);
			}

			titles
		}
		None => String::new(),
	}
}
