use graphql_client::{GraphQLQuery, Response};
use poise::serenity_prelude as serenity;

use super::{anilist_query, clean_html, send_deletable};

#[derive(GraphQLQuery)]
#[graphql(schema_path = "schema.json", query_path = "queries/SearchUser.graphql")]
struct SearchUser;

/// Searches for an AniList account by name
#[poise::command(slash_command, rename = "user")]
pub(crate) async fn find_user(
	ctx: crate::Context<'_>,
	#[description = "Username to search"] name: String,
) -> Result<(), crate::Error> {
	let request_body: graphql_client::QueryBody<search_user::Variables> =
		SearchUser::build_query(search_user::Variables { search: name });
	let response_body: Response<search_user::ResponseData> =
		anilist_query(&request_body).await?.json().await?;

	let user = match response_body.data {
		Some(data) => data.user.unwrap(),
		None => {
			ctx.say("User not found").await?;
			return Ok(());
		}
	};

	let user_stats = match &user.statistics {
		Some(stats) => {
			let minutes_watched = stats.anime.as_ref().unwrap().minutes_watched;
			let minutes_watched = if minutes_watched > 0 {
				let minutes_watched = minutes_watched as f64;
				// Use the en quad space after days watched to not get stripped by Discord
				format!("Days watched: {:.1}  ", minutes_watched / (24 * 60) as f64)
			} else {
				"".to_string()
			};

			let chapters_read = stats.manga.as_ref().unwrap().chapters_read;
			let chapter_read = if chapters_read > 0 {
				format!("Chapters read: {}", chapters_read)
			} else {
				"".to_string()
			};

			format!("{}{}", minutes_watched, chapter_read)
		}
		None => "".to_string(),
	};

	let bio = match &user.about {
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

	let user_url = &user.site_url.unwrap();
	let embed = serenity::CreateEmbed::new()
		.author(
			serenity::CreateEmbedAuthor::new(&user.name)
				.icon_url("https://anilist.co/img/icons/favicon-32x32.png")
				.url(user_url),
		)
		.thumbnail(&user.avatar.unwrap().large.unwrap())
		.description(bio)
		.color(3447003)
		.footer(serenity::CreateEmbedFooter::new(user_stats));

	send_deletable(
		ctx,
		poise::CreateReply {
			content: Some(format!("<{}>", &user_url)),
			embeds: vec![embed],
			reply: true,
			..Default::default()
		},
	)
	.await?;
	Ok(())
}
