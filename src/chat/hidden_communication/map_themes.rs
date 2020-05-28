use super::wait_for_message;
use crate::{
    async_manager,
    entity_manager::EntityManager,
    error::*,
    options::{AUTOPLAY_MAP_THEMES, MAP_THEME_VOLUME},
    players::{MediaPlayer, Player, PlayerTrait, YoutubePlayer},
};
use classicube_helpers::{tab_list::remove_color, CellGetSet};
use futures::{future::RemoteHandle, prelude::*};
use log::{debug, info, warn};
use std::{cell::Cell, time::Duration};
use url::Url;

thread_local!(
    static LISTENER: Cell<Option<RemoteHandle<()>>> = Default::default();
);

pub fn start_listening() {
    let (f, remote_handle) = async {
        listen_loop().await;
    }
    .remote_handle();

    async_manager::spawn_local_on_main_thread(f);

    LISTENER.with(move |cell| {
        cell.set(Some(remote_handle));
    });
}

pub fn stop_listening() {
    LISTENER.with(move |cell| {
        cell.set(None);
    });
}

fn is_map_theme_message(message: &str) -> bool {
    let m = remove_color(message).to_lowercase();

    m.starts_with("map theme: ") || m.starts_with("map theme song: ")
}

pub async fn listen_loop() {
    loop {
        let message = wait_for_message().await;
        if is_map_theme_message(&message) {
            debug!("got map_theme url first part {:?}", message);

            let mut parts: Vec<String> = Vec::new();
            parts.push(message);

            let timeout_result = async_manager::timeout(Duration::from_secs(1), async {
                loop {
                    let message = wait_for_message().await;
                    if message.starts_with("> &f") {
                        parts.push(message[4..].to_string());
                    } else {
                        debug!("stopping because of other message {:?}", message);
                        break;
                    }
                }
            })
            .await;

            if timeout_result.is_none() {
                debug!("stopping because of timeout");
            }

            let full_message: String = parts.join("");
            let full_message = remove_color(full_message);

            info!("map_theme {:?}", full_message);

            async_manager::spawn_local_on_main_thread(async move {
                match handle_map_theme_url(full_message).await {
                    Ok(_) => {}

                    Err(e) => {
                        warn!("map_theme listen_loop: {}", e);
                    }
                }
            });
        }
    }
}

thread_local!(
    pub static CURRENT_MAP_THEME: Cell<Option<usize>> = Default::default();
);

pub fn on_new_map_loaded() {
    CURRENT_MAP_THEME.set(None);
}

async fn handle_map_theme_url(message: String) -> Result<()> {
    if !AUTOPLAY_MAP_THEMES.get()? {
        return Ok(());
    }

    let regex = regex::Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();

    let match_ = regex.find(&message).chain_err(|| "regex find url")?;
    let url = match_.as_str();
    let url = Url::parse(url)?;

    debug!("map_theme got {:?}", url);

    let volume = MAP_THEME_VOLUME.get()?;
    let player = match YoutubePlayer::from_url(&url) {
        Ok(mut player) => {
            player.volume = volume;
            Player::Youtube(player)
        }
        Err(youtube_error) => match MediaPlayer::from_url(&url) {
            Ok(mut player) => {
                player.volume = volume;
                Player::Media(player)
            }
            Err(media_error) => {
                bail!(
                    "couldn't create any player for url {:?}: {}, {}",
                    url,
                    youtube_error,
                    media_error
                );
            }
        },
    };

    let entity_id = if let Some(entity_id) = CURRENT_MAP_THEME.get() {
        EntityManager::entity_play_player(player, entity_id)?;
        entity_id
    } else {
        // 1 fps, 1x1 resolution
        let entity_id = EntityManager::create_entity_player(player, 1, false, Some((1, 1)))?;
        EntityManager::with_by_entity_id(entity_id, |entity| {
            entity.set_scale(0.0);

            Ok(())
        })?;

        entity_id
    };

    CURRENT_MAP_THEME.set(Some(entity_id));

    // set quiet volume, and don't send to other players
    EntityManager::with_by_entity_id(entity_id, |entity| {
        entity.player.set_should_send(false);
        entity.player.set_global_volume(true)?;

        Ok(())
    })?;

    Ok(())
}
