use serenity::all::{CommandDataOptionValue, Context, GuildId, GuildPagination};

#[derive(Debug)]
pub struct Config {
    interval: String,
    time: String,
}

pub async fn import_config(guild_id: GuildId, value: CommandDataOptionValue) {
    println!("guild id {} value {}", guild_id, value.as_str().unwrap())
}

pub async fn read_config(_guild_id: GuildId) -> anyhow::Result<Config> {
    // For now, return dummy values regardless of guild_id
    Ok(Config {
        interval: "daily".to_string(),
        time: "09:00".to_string(),
    })
}

pub async fn get_guilds(ctx: Context) -> anyhow::Result<Vec<GuildId>> {
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
