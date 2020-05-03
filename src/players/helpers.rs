use super::{MediaPlayer, Player, PlayerTrait, YoutubePlayer};
use crate::{async_manager::AsyncManager, chat::ENTITIES, entity_manager::EntityManager, error::*};
use classicube_helpers::OptionWithInner;
use classicube_sys::ENTITIES_SELF_ID;
use std::time::Duration;

pub async fn start_update_loop(entity_id: usize) {
    let result = start_loop(entity_id).await;

    if let Err(e) = result {
        log::warn!("start_update_loop {} {}", entity_id, e);
    }
}

async fn start_loop(entity_id: usize) -> Result<()> {
    loop {
        // update volume via distance
        EntityManager::with_by_entity_id(entity_id, |entity| {
            if let Some(browser) = &entity.browser {
                let maybe_my_pos = ENTITIES
                    .with_inner(|entities| {
                        let me = entities.get(ENTITIES_SELF_ID as _)?;

                        Some(me.get_position())
                    })
                    .flatten();

                if let Some(my_pos) = maybe_my_pos {
                    let entity_pos = entity.entity.Position;

                    let percent = (entity_pos - my_pos).length_squared().sqrt() / 30f32;
                    let percent = (1.0 - percent).max(0.0).min(1.0);

                    entity.player.set_volume(&browser, percent)?;
                }
            }

            Ok(())
        })?;

        enum Kind {
            Youtube,
            Media,
        }
        // update time
        let (browser, kind) = EntityManager::with_by_entity_id(entity_id, |entity| {
            Ok(match &entity.player {
                Player::Media(_) => (entity.browser.as_ref().cloned(), Kind::Media),
                Player::Youtube(_) => (entity.browser.as_ref().cloned(), Kind::Youtube),

                _ => {
                    bail!("not supported");
                }
            })
        })?;

        if let Some(browser) = browser {
            let time = match kind {
                Kind::Media => MediaPlayer::get_real_time(&browser).await,
                Kind::Youtube => YoutubePlayer::get_real_time(&browser).await,
            };

            if let Ok(time) = time {
                EntityManager::with_by_entity_id(entity_id, move |entity| {
                    match &mut entity.player {
                        Player::Media(player) => {
                            player.time = time;
                        }
                        Player::Youtube(player) => {
                            player.time = time;
                        }

                        _ => {
                            bail!("not supported");
                        }
                    }
                    Ok(())
                })?;
            }
        }

        AsyncManager::sleep(Duration::from_millis(64)).await;
    }
}
