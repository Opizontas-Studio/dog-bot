use serenity::all::*;

use crate::error::BotError;

fn get_direct_children_channels<'a, 'b>(
    channels: &'a [&GuildChannel],
    channel: &'b GuildChannel,
) -> Vec<&'a GuildChannel> {
    channels
        .iter()
        .filter(|&c| c.parent_id == Some(channel.id))
        .cloned()
        .collect()
}

pub async fn get_children_channels(
    http: &Http,
    guild: &Guild,
    channel: &GuildChannel,
) -> Result<Vec<GuildChannel>, BotError> {
    let channels = guild.channels(http).await?;
    let channels = channels.values().collect::<Vec<_>>();
    let children = std::iter::successors(Some(vec![channel]), |cs| {
        Some(
            cs.iter()
                .flat_map(|c| get_direct_children_channels(&channels, c))
                .collect(),
        )
        .filter(|children: &Vec<_>| !children.is_empty())
    })
    .flatten()
    .cloned()
    .collect();
    Ok(children)
}
