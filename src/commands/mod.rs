use std::{collections::HashMap, time::Duration};

use html2md::{TagHandler, TagHandlerFactory};
use poise::{
	serenity_prelude::{self as serenity, futures::StreamExt},
	CreateReply,
};

pub(crate) mod character;
pub(crate) mod media;
pub(crate) mod staff;
pub(crate) mod studio;
pub(crate) mod user;

pub(crate) async fn anilist_query<V: serde::Serialize>(
	query: &graphql_client::QueryBody<V>,
) -> Result<reqwest::Response, crate::Error> {
	let client = reqwest::Client::new();
	let res = client
		.post("https://graphql.anilist.co")
		.json(query)
		.send()
		.await?;

	Ok(res)
}

pub(crate) fn clean_html(text: &str) -> String {
	let text = text.replace("\n\n", "<br>");

	let mut tag_factory: HashMap<String, Box<dyn TagHandlerFactory>> = HashMap::new();
	tag_factory.insert(String::from("span"), Box::new(SpoilerStripperFactory {}));
	let text = html2md::parse_html_custom(&text, &tag_factory);

	let text = text.replace("---", "");
	let text = text.replace("#", "");

	text
}

pub(crate) async fn send_deletable<'a>(
	context: crate::Context<'a>,
	msg: CreateReply,
) -> Result<(), serenity::Error> {
	let msg = context.send(msg).await?;

	let bot_reaction = &msg
		.message()
		.await?
		.react(
			&context,
			serenity::ReactionType::Unicode(String::from("❌")),
		)
		.await?;

	let mut collector = serenity::ReactionCollector::new(context)
		.timeout(Duration::from_secs(20))
		.message_id(msg.message().await?.id)
		.author_id(context.author().id)
		.filter(move |reaction| reaction.emoji.unicode_eq("❌"))
		.stream();

	loop {
		match collector.next().await {
			Some(_) => {
				msg.delete(context).await?;
				break;
			}
			None => {
				bot_reaction.delete(context).await?;
				break;
			}
		}
	}

	Ok(())
}

struct SpoilerStripperFactory;

impl TagHandlerFactory for SpoilerStripperFactory {
	fn instantiate(&self) -> Box<dyn TagHandler> {
		return Box::new(SpoilerStripper::default());
	}
}

#[derive(Default)]
pub(crate) struct SpoilerStripper;

impl TagHandler for SpoilerStripper {
	fn handle(&mut self, _tag: &html2md::Handle, _printer: &mut html2md::StructuredPrinter) {}

	fn skip_descendants(&self) -> bool {
		true
	}

	fn after_handle(&mut self, _printer: &mut html2md::StructuredPrinter) {}
}
