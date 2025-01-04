use serenity::all::{CommandDataOptionValue, GuildId};

pub async fn import(guild_id: GuildId, value: CommandDataOptionValue) {
    println!("guild id {} value {}", guild_id, value.as_str().unwrap())
}
