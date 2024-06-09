use graphql_client::{GraphQLQuery, Response};
use poise::serenity_prelude as serenity;

use super::{anilist_query, clean_html, send_deletable};

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "schema.json",
	query_path = "queries/SearchCharacter.graphql"
)]
struct SearchCharacter;

/// Searches for a character by name
#[poise::command(slash_command, rename = "character")]
pub(crate) async fn find_character(
	ctx: crate::Context<'_>,
	#[description = "Name to search"] name: String,
) -> Result<(), crate::Error> {
	let request_body = SearchCharacter::build_query(search_character::Variables { search: name });
	let response_body: Response<search_character::ResponseData> =
		anilist_query(&request_body).await?.json().await?;
	let character = match response_body.data {
		Some(data) => match data.character {
			Some(character) => character,
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

	let bio = match &character.description {
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

	let character_url = &character.site_url.unwrap();
	let embed = serenity::CreateEmbed::new()
		.author(
			serenity::CreateEmbedAuthor::new(&character.name.unwrap().full.unwrap())
				.icon_url("https://anilist.co/img/icons/favicon-32x32.png")
				.url(character_url),
		)
		.thumbnail(&character.image.unwrap().large.unwrap())
		.description(bio)
		.color(3447003);

	send_deletable(
		ctx,
		poise::CreateReply {
			content: Some(format!("<{}>", &character_url)),
			embeds: vec![embed],
			reply: true,
			..Default::default()
		},
	)
	.await?;

	Ok(())
}
