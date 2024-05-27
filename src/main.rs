use dotenvy::dotenv;
use poise::serenity_prelude as serenity;

mod commands;

struct Data {}
pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
	dotenv().ok();

	let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is required");
	let intents = serenity::GatewayIntents::non_privileged();

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			commands: vec![
				help(),
				commands::media::find_anime(),
				commands::media::find_manga(),
				commands::character::find_character(),
				commands::staff::find_staff(),
				commands::studio::find_studio(),
				commands::user::find_user(),
			],
			..Default::default()
		})
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(Data {})
			})
		})
		.build();

	let client = serenity::ClientBuilder::new(token, intents)
		.framework(framework)
		.await;

	client
		.unwrap()
		.start()
		.await
		.expect("Client failed to start.");
}

#[poise::command(slash_command)]
async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
	let configuration = poise::builtins::HelpConfiguration {
		ephemeral: true,
		..Default::default()
	};

	poise::builtins::help(ctx, command.as_deref(), configuration).await?;
	Ok(())
}
