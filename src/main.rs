mod weather;

use anyhow::Context as _;
use serenity::all::{
    ActivityData, CommandOptionType, CreateEmbed, CreateEmbedAuthor, GuildId, Interaction, Mention,
    OnlineStatus,
};
use serenity::async_trait;
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::info;

struct Bot {
    weather_api_key: String,
    client: reqwest::Client,
    discord_guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_presence(
            Some(ActivityData::custom("*BGM of The Terminator*")),
            OnlineStatus::Online,
        );

        let commands = vec![
            CreateCommand::new("weather")
                .description("Display the weather")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "place",
                        "City to lookup forecast",
                    )
                    .required(true),
                ),
            CreateCommand::new("ぬるぽ").description("ｶﾞｯ"),
            CreateCommand::new("config").description("Open config GUI"),
            CreateCommand::new("import-config")
                .description("Import config from config code")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "config",
                        "Config code that is shown underneath the config",
                    )
                    .required(true),
                ),
            CreateCommand::new("add")
                .description("Add a team member")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::User, "user", "User to add")
                        .required(true),
                ),
            CreateCommand::new("remove")
                .description("Remove a team member")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::User, "user", "User to remove")
                        .required(true),
                ),
            CreateCommand::new("progress")
                .description("Report the progress")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::String, "progress", "Progress to report")
                        .required(true),
                ),
        ];

        let commands = &self
            .discord_guild_id
            .set_commands(&ctx.http, commands)
            .await
            .unwrap();

        info!(
            "Registered commands: {:#?}",
            commands.iter().map(|c| c.name.clone()).collect::<Vec<_>>()
        );
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let data = match command.data.name.as_str() {
                "weather" => CreateInteractionResponseMessage::new().content({
                    let argument = command
                        .data
                        .options
                        .iter()
                        .find(|opt| opt.name == "place")
                        .cloned();

                    let value = argument.unwrap().value;
                    let place = value.as_str().unwrap();

                    let result =
                        weather::get_forecast(place, &self.weather_api_key, &self.client).await;

                    match result {
                        Ok((location, forecast)) => {
                            format!("Forecast: {} in {}", forecast.headline.overview, location)
                        }
                        Err(err) => {
                            format!("Err: {}", err)
                        }
                    }
                }),
                "ぬるぽ" => CreateInteractionResponseMessage::new().content(
                    "
ㅤ （　・∀・）　 |　|　ｶﾞｯ
　と　　　　）　|　|
　　 Ｙ　/ノ　　人
　　　 /　）　 < 　>__Λ∩
　 ＿/し'　／／. Ｖ｀Д´）/ ←お前
　（＿フ彡　　　　　　/"
                ),
                "config" => CreateInteractionResponseMessage::new().embed(
                    CreateEmbed::new()
                        .author(CreateEmbedAuthor::new("Slacker Slayer Config"))
                        .description("Slacker Slayer GUI config"),
                ),
                "import-config" => CreateInteractionResponseMessage::new().content({
                    let argument = command
                        .data
                        .options
                        .iter()
                        .find(|opt| opt.name == "config")
                        .cloned();

                    let value = argument.unwrap().value;
                    format!("Import {}", value.as_str().unwrap())
                }),
                "add" => CreateInteractionResponseMessage::new().content({
                    let argument = command
                        .data
                        .options
                        .iter()
                        .find(|opt| opt.name == "user")
                        .cloned();

                    let value = argument.unwrap().value.as_user_id().unwrap();
                    format!("Add: {}", Mention::from(value))
                }),
                "remove" => CreateInteractionResponseMessage::new()
                    .content({
                        let argument = command
                            .data
                            .options
                            .iter()
                            .find(|opt| opt.name == "user")
                            .cloned();

                        let value = argument.unwrap().value;
                        format!("Remove: {:?}", value)
                    })
                    .ephemeral(true),
                "progress" => CreateInteractionResponseMessage::new()
                    .content({
                        let argument = command
                            .data
                            .options
                            .iter()
                            .find(|opt| opt.name == "progress")
                            .cloned();

                        let value = argument.unwrap().value;
                        format!("You reported the progress: {}", value.as_str().unwrap())
                    })
                    .ephemeral(true),
                command => unreachable!("Unknown command: {}", command),
            };

            let builder = CreateInteractionResponse::Message(data);

            if let Err(why) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command: {why}");
            }
        }
    }
}

async fn hourly_deadline_check() {
    loop {
        println!("Hourly message");
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Run the hourly deadline check
    tokio::spawn(hourly_deadline_check());

    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let weather_api_key = secret_store
        .get("WEATHER_API_KEY")
        .context("'WEATHER_API_KEY' was not found")?;

    let discord_guild_id = secret_store
        .get("DISCORD_GUILD_ID")
        .context("'DISCORD_GUILD_ID' was not found")?;

    let client = get_client(
        &discord_token,
        &weather_api_key,
        discord_guild_id.parse().unwrap(),
    )
    .await;
    Ok(client.into())
}

pub async fn get_client(
    discord_token: &str,
    weather_api_key: &str,
    discord_guild_id: u64,
) -> Client {
    // Set gateway intents, which decides what events the bot will be notified about.
    // Here we don't need any intents so empty
    let intents = GatewayIntents::empty();

    Client::builder(discord_token, intents)
        .event_handler(Bot {
            weather_api_key: weather_api_key.to_owned(),
            client: reqwest::Client::new(),
            discord_guild_id: GuildId::new(discord_guild_id),
        })
        .await
        .expect("Err creating client")
}
