use graphql_client::{GraphQLQuery, Response};
use poise::serenity_prelude as serenity;

use super::{anilist_query, clean_html, send_deletable};

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "schema.json",
	query_path = "queries/SearchMedia.graphql"
)]
struct SearchMedia;

/// Searches for an anime by name
#[poise::command(slash_command, rename = "anime")]
pub(crate) async fn find_anime(
	ctx: crate::Context<'_>,
	#[description = "Title to search"] name: String,
) -> Result<(), crate::Error> {
	find_media(ctx, name, search_media::MediaType::ANIME).await
}

/// Searches for a manga by name
#[poise::command(slash_command, rename = "manga")]
pub(crate) async fn find_manga(
	ctx: crate::Context<'_>,
	#[description = "Title to search"] name: String,
) -> Result<(), crate::Error> {
	find_media(ctx, name, search_media::MediaType::MANGA).await
}

async fn find_media(
	ctx: crate::Context<'_>,
	name: String,
	media_type: search_media::MediaType,
) -> Result<(), crate::Error> {
	let request_body = SearchMedia::build_query(search_media::Variables {
		search: name,
		type_: media_type,
	});
	let response_body: Response<search_media::ResponseData> =
		anilist_query(&request_body).await?.json().await?;

	let media = match response_body.data {
		Some(data) => data.media.unwrap(),
		None => {
			ctx.say("Not found").await?;
			return Ok(());
		}
	};

	let media_stats = if media.status.is_some() || media.mean_score.is_some() {
		let score = match &media.mean_score {
			// Use the en quad space after days watched to not get stripped by Discord
			Some(score) => format!("Score: {}%  ", score),
			None => "".to_string(),
		};

		let mut status = match media.status {
			Some(status) => match status {
				search_media::MediaStatus::RELEASING => "Releasing",
				search_media::MediaStatus::NOT_YET_RELEASED => "Not yet released",
				search_media::MediaStatus::FINISHED => "Finished",
				search_media::MediaStatus::CANCELLED => "Cancelled",
				search_media::MediaStatus::HIATUS => "Hiatus",
				search_media::MediaStatus::Other(_unknown_status) => "",
			},
			None => "",
		}
		.to_string();

		if status.len() > 0 {
			status = format!("Status: {}", status);
		}

		format!("{}{}", score, status)
	} else {
		"".to_string()
	};

	let description = match &media.description {
		Some(desc) => {
			let mut desc = clean_html(desc);
			// Keep descriptions short, but if it has <100 characters left, just let it finish
			if desc.len() > 400 && desc.len() >= 500 {
				desc = format!("{}...", desc.split_at(400).0);
			}
			desc
		}
		None => "".to_string(),
	};

	let media_url = &media.site_url.unwrap();
	let embed = serenity::CreateEmbed::new()
		.author(
			serenity::CreateEmbedAuthor::new(&media.title.unwrap().romaji.unwrap())
				.icon_url("https://anilist.co/img/icons/favicon-32x32.png")
				.url(media_url),
		)
		.thumbnail(&media.cover_image.unwrap().extra_large.unwrap())
		.description(description)
		.color(3447003)
		.footer(serenity::CreateEmbedFooter::new(media_stats));

	send_deletable(
		ctx,
		poise::CreateReply {
			content: Some(format!("<{}>", &media_url)),
			embeds: vec![embed],
			reply: true,
			..Default::default()
		},
	)
	.await?;

	Ok(())
}
