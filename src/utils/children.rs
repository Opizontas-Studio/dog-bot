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
    let mut channels = vec![vec![channel.to_owned()]];

    loop {
        let children: Vec<GuildChannel> = channels
            .last()
            .unwrap()
            .iter()
            .flat_map(|c| get_direct_children_channels(guild, c))
            .collect();
        if children.is_empty() {
            break;
        }
        channels.push(children);
    }
    channels.into_iter().flatten().collect()
}
