use serenity::{
    all::{ChannelId, CommandDataOptionValue, GuildId, GuildPagination, Message, UserId},
    builder::GetMessages,
};

#[derive(Debug)]
pub struct Config {
    pub interval: String,    // daily, weekly, monthly
    pub time: u8,            // 0 to 23
    pub weekday: Option<u8>, // 1-7 for Monday-Sunday
    pub day: Option<u8>,     // 1-31 for day of month
    pub channel_id: Option<ChannelId>,
}

#[derive(Debug)]
pub struct Users {
    pub id: UserId,
    pub name: String,
    pub avatar_url: Option<String>,
}

pub async fn get_guilds() -> anyhow::Result<Vec<GuildId>> {
    let ctx = crate::Bot::get_context().await?;

    let mut last_guild_id = GuildId::new(1);
    let mut guild_ids: Vec<GuildId> = vec![];

    loop {
        if let Ok(guilds) = ctx
            .http
            .get_guilds(Some(GuildPagination::After(last_guild_id)), Some(100))
            .await
        {
            if guilds.is_empty() {
                break;
            }

            guild_ids = [guild_ids, guilds.iter().map(|guild| guild.id).collect()].concat();

            if guilds.len() < 100 {
                break;
            }

            last_guild_id = guilds.last().unwrap().id
        }
    }

    Ok(guild_ids)
}

pub async fn get_bot_messages(guild_id: GuildId) -> anyhow::Result<Vec<Message>> {
    // Get the context from the global state
    let ctx = crate::Bot::get_context().await?;
    let bot_id = ctx.http.get_current_user().await.unwrap().id;

    // Get all channels in the guild
    let channels = guild_id.channels(&ctx).await?;

    let mut bot_messages = Vec::new();

    // Search through all channels for messages from the bot
    for (_channel_id, channel) in channels {
        // Get messages in the channel
        let messages = channel
            .messages(&ctx, GetMessages::new().limit(100))
            .await?;

        // Filter messages by author ID
        for msg in messages {
            if msg.author.id == bot_id {
                bot_messages.push(msg);
            }
        }
    }

    println!("{:?}", bot_messages);

    Ok(bot_messages)
}

pub async fn get_config_code(guild_id: GuildId, bot_id: UserId) -> anyhow::Result<String> {
    let messages = get_bot_messages(guild_id).await.unwrap();

    // Look for messages with embeds titled "Config"
    for msg in messages {
        for embed in msg.embeds {
            if let Some(title) = &embed.title {
                if title == "Config" && msg.author.id == bot_id {
                    // For now, just print the config code
                    println!("Found config message: {:?}", embed);

                    if let Some(description) = &embed.description {
                        return Ok(description.to_string());
                    }
                }
            }
        }
    }

    // If we get here, no config was found
    println!("Config not found.");
    anyhow::bail!("Config not found")
}

/* async fn get_system_channel(guild_id: GuildId) -> anyhow::Result<ChannelId> {
    let ctx = crate::Bot::get_context().await?;

    let guild = guild_id.to_partial_guild(&ctx.http).await?;
    let system_channel = guild
        .system_channel_id
        .ok_or_else(|| anyhow::anyhow!("System channel not found for guild {}", guild_id))?;

    Ok(system_channel)
}

pub async fn get_project_members(ctx: Context, guild_id: GuildId, bot_id: UserId) -> anyhow::Result<Vec<Users>> {
    const config = parse_config_code(read_config_code(guild_id, bot_id));
}

pub async fn get_guild_members(guild_id: GuildId) -> anyhow::Result<Vec<Users>> {
    let ctx = crate::Bot::get_context().await?;

    let members = guild_id.members(&ctx.http, None, None).await?;

    println!("members: {:?}", members);

    let users = members
        .into_iter()
        .map(|member| Users {
            id: member.user.id,
            name: member.user.name.clone(),
            avatar_url: member.user.avatar_url(),
        })
        .collect();

    Ok(users)
} */

pub async fn import_config(guild_id: GuildId, value: CommandDataOptionValue) {
    println!("guild id {} value {}", guild_id, value.as_str().unwrap())
}

pub async fn parse_config_code(code: String) -> Config {
    // Format:
    // daily: "ssc.daily.09:00.channel_id"
    // weekly: "ssc.weekly.1.09:00.channel_id" (1 = Monday)
    // monthly: "ssc.monthly.15.09:00.channel_id" (15 = 15th day of month)
    let parts: Vec<&str> = code.trim().split('.').collect();

    let interval = parts[1].to_string();
    if (parts.len() < 3 || parts[0] != "ssc")
        || (interval != "daily" && parts.len() < 3)
        || (interval != "weekly" && parts.len() < 4)
        || (interval != "monthly" && parts.len() < 4)
    {
        // If invalid format, return default config
        return Config {
            interval: "daily".to_string(),
            time: 9,
            weekday: None,
            day: None,
            channel_id: None,
        };
    }

    Config {
        interval: parts[1].to_string(),
        time: parts[2].parse().unwrap_or(9),
        weekday: parts[3].parse().ok(),
        day: parts[4].parse().ok(),
        channel_id: parts.get(5).and_then(|s| s.parse().ok()),
    }
}

/* pub async fn config_gui(guild_id: GuildId) -> anyhow::Result<()> {
    // Get the context from the global state
    let ctx = crate::Bot::get_context().await?;

    guild_id.channels(http);

    // Get the system channel to send message
    if let Some(channel) = guild_id.system_channel(&ctx).await? {
        channel.send_message(&ctx, CreateMessage::new())
    }
} */
