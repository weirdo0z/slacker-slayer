mod util;

use anyhow::Context as _;
use once_cell::sync::OnceCell;
use serenity::all::{
    ActivityData, CommandDataOptionValue, CommandInteraction, CommandOptionType, CreateEmbed,
    CreateEmbedAuthor, CreateMessage, GatewayIntents, GuildId, Interaction, Mention, OnlineStatus,
    UserId,
};
use serenity::async_trait;
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

static BOT: OnceCell<Arc<Bot>> = OnceCell::new();

#[derive(Clone)]
pub struct Bot {
    pub discord_guild_id: GuildId,
    pub ctx: Arc<RwLock<Option<Context>>>,
    pub bot_id: UserId,
}

impl Bot {
    pub async fn get_context() -> anyhow::Result<Context> {
        let bot = BOT
            .get()
            .ok_or_else(|| anyhow::anyhow!("Bot not initialized"))?
            .ctx
            .read()
            .await;
        let ctx = bot
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Context not available"))?;
        Ok(ctx.to_owned())
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Store context when bot is ready
        *self.ctx.write().await = Some(ctx.to_owned());
        BOT.get_or_init(|| Arc::new(self.to_owned()));
        info!("{} is connected!", ready.user.name);
        ctx.set_presence(
            Some(ActivityData::custom("*BGM of The Terminator*")),
            OnlineStatus::Online,
        );

        let commands = vec![
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
            CreateCommand::new("assign")
                .description("Assign a task to a team member")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::User, "user", "User to assign")
                        .required(true),
                )
                .add_option(
                    CreateCommandOption::new(CommandOptionType::String, "task", "Task to assign")
                        .required(true),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "deadline",
                        "Deadline of the task (e.g., 'March 14th 16:32', 'tomorrow 3 am')",
                    )
                    .required(true),
                ),
            CreateCommand::new("members").description("Show the list of team members"),
            CreateCommand::new("progress")
                .description("Report the progress")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "progress",
                        "Progress to report",
                    )
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
            commands
                .iter()
                .map(|c| c.name.to_owned())
                .collect::<Vec<_>>()
        );
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = &interaction {
            fn get_option(command: CommandInteraction, name: &str) -> CommandDataOptionValue {
                command
                    .data
                    .options
                    .iter()
                    .find(|opt| opt.name == name)
                    .cloned()
                    .unwrap()
                    .value
            }

            fn create_project_members_embed() -> CreateEmbed {
                CreateEmbed::new().title("Members").description("members")
            }

            let data = match command.data.name.as_str() {
                "ぬるぽ" => CreateInteractionResponseMessage::new().content(
                    "
ㅤ （　・∀・）　 |　|　ｶﾞｯ
　と　　　　）　|　|
　　 Ｙ　/ノ　　人
　　　 /　）　 < 　>__Λ∩
　 ＿/し'　／／. Ｖ｀Д´）/ ←お前
　（＿フ彡　　　　　　/",
                ),
                "config" => CreateInteractionResponseMessage::new().embed({
                    CreateEmbed::new()
                        .author(CreateEmbedAuthor::new("Slacker Slayer Config"))
                        .description("Slacker Slayer GUI config")
                }),
                "import-config" => CreateInteractionResponseMessage::new().embed({
                    let value = get_option(command.to_owned(), "config");

                    let guild_id = &interaction.as_command().unwrap().guild_id.unwrap();
                    util::import_config(*guild_id, value.to_owned()).await;

                    CreateEmbed::new()
                        .author(CreateEmbedAuthor::new("Slacker Slayer Config"))
                        .description(format!(
                            "Imported \"{}\" and now the setting is:",
                            value.as_str().unwrap(),
                        ))
                }),
                "add" => CreateInteractionResponseMessage::new()
                    .content({
                        let value = get_option(command.to_owned(), "user");
                        let user = value.as_user_id().unwrap();

                        format!("Added {}.", Mention::from(user))
                    })
                    .embed(create_project_members_embed()),
                "remove" => CreateInteractionResponseMessage::new()
                    .content({
                        let value = get_option(command.to_owned(), "user");
                        let user = value.as_user_id().unwrap();

                        format!("Removed {}.", Mention::from(user))
                    })
                    .embed(create_project_members_embed()),
                "assign" => CreateInteractionResponseMessage::new().content({
                    let (user, task, deadline) = (
                        get_option(command.to_owned(), "user"),
                        get_option(command.to_owned(), "task"),
                        get_option(command.to_owned(), "deadline"),
                    );

                    format!(
                        "Assigned {} to {} by {}.",
                        Mention::from(user.as_user_id().unwrap()),
                        task.as_str().unwrap(),
                        deadline.as_str().unwrap()
                    )
                }),
                "members" => {
                    /*let guild_id = &interaction.as_command().unwrap().guild_id.unwrap();
                    let members = util::get_project_members(ctx.to_owned(), *guild_id, bot_id).await;

                    println!("{:?}", &members);

                    CreateInteractionResponseMessage::new()
                        .embed({
                            match &members {
                                Ok(members) => CreateEmbed::new()
                                    .title("Current project members")
                                    .description(
                                        members
                                            .iter()
                                            .map(|user| {
                                                if let Some(avatar) = &user.avatar_url {
                                                    format!("[{}]({})", user.name, avatar)
                                                } else {
                                                    user.name.to_owned()
                                                }
                                            })
                                            .collect::<Vec<String>>()
                                            .join("\n"),
                                    ),
                                Err(_) => CreateEmbed::new()
                                    .title("Oops,")
                                    .description("member not found."),
                            }
                        })
                        .ephemeral(members.is_err())*/
                    CreateInteractionResponseMessage::new().embed(create_project_members_embed())
                }
                "progress" => CreateInteractionResponseMessage::new()
                    .content({
                        let value = get_option(command.to_owned(), "progress");

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

async fn hourly_deadline_check(wrapped_ctx: Arc<RwLock<Option<Context>>>) {
    // Wait for context to be available
    loop {
        if wrapped_ctx.read().await.is_some() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;

        let ctx = wrapped_ctx.read().await.as_ref().unwrap().to_owned();
        let guild_ids = match util::get_guilds().await {
            Ok(ids) => ids,
            Err(e) => {
                println!("Failed to get guilds: {}", e);
                break;
            }
        };

        for guild_id in guild_ids {
            let config = util::parse_config_code(
                util::get_config_code(guild_id, ctx.http.get_current_user().await.unwrap().id)
                    .await
                    .unwrap(),
            )
            .await;

            println!("Guild {}: Config {:?}", guild_id, config);
        }
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let discord_guild_id = secret_store
        .get("DISCORD_GUILD_ID")
        .context("'DISCORD_GUILD_ID' was not found")?;

    let bot_id = secret_store
        .get("BOT_ID")
        .context("'BOT_ID' was not found")?;

    // Create shared context
    let shared_ctx = Arc::new(RwLock::new(None));

    // Create client with shared context
    let client = {
        let intents = GatewayIntents::GUILDS;
        let bot = Bot {
            discord_guild_id: GuildId::new(discord_guild_id.parse().unwrap()),
            ctx: shared_ctx.to_owned(),
            bot_id: UserId::new(bot_id.parse().unwrap()),
        };

        Client::builder(&discord_token, intents)
            .event_handler(bot)
            .await
            .expect("Err creating client")
    };

    // Run the hourly deadline check with shared context
    tokio::spawn(hourly_deadline_check(shared_ctx));

    Ok(client.into())
}
