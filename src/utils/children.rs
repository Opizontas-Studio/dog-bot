use serenity::all::*;

fn get_direct_children_channels(guild: &Guild, channel: &GuildChannel) -> Vec<GuildChannel> {
    guild
        .channels
        .values()
        .filter(|&c| c.parent_id == Some(channel.id))
        .cloned()
        .collect()
}

pub fn get_all_children_channels(guild: &Guild, channel: &GuildChannel) -> Vec<GuildChannel> {
    std::iter::successors(Some(vec![channel.to_owned()]), |cs| {
        Some(
            cs.iter()
                .flat_map(|c| get_direct_children_channels(guild, c))
                .collect(),
        )
        .filter(|children: &Vec<_>| !children.is_empty())
    })
    .flatten()
    .collect()
}
