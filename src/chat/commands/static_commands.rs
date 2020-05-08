use super::helpers::*;
use crate::{
    async_manager::AsyncManager, chat::PlayerSnapshot, entity_manager::EntityManager, error::*,
};
use clap::{App, Arg, ArgMatches};

// static commands not targetted at a specific entity
pub fn add_commands(app: App<'static, 'static>) -> App<'static, 'static> {
    // url has to be multiple because urls can be chopped in half by
    // line continuations, so we join the parts together as a hack

    app.subcommand(
        App::new("create")
            .about("Creates a new screen")
            .arg(Arg::with_name("url").multiple(true)),
    )
    .subcommand(
        App::new("closeall")
            .aliases(&["removeall", "clearall"])
            .about("Close all screens"),
    )
}

pub async fn handle_command(
    player: &PlayerSnapshot,
    matches: &ArgMatches<'static>,
) -> Result<bool> {
    match matches.subcommand() {
        ("create", Some(matches)) => {
            let parts = matches.values_of_lossy("url").unwrap_or_default();

            if parts.is_empty() {
                let entity_id = EntityManager::create_entity("https://www.classicube.net/")?;
                EntityManager::with_by_entity_id(entity_id, |entity| {
                    move_entity(entity, player);

                    Ok(())
                })?;
            } else {
                let url: String = parts.join("");

                let entity_id = EntityManager::create_entity(&url)?;
                EntityManager::with_by_entity_id(entity_id, |entity| {
                    move_entity(entity, player);

                    Ok(())
                })?;
            }

            Ok(true)
        }

        ("closeall", Some(_matches)) => {
            AsyncManager::spawn_local_on_main_thread(async {
                let _ignore_error = EntityManager::remove_all_entities().await;
            });

            Ok(true)
        }

        _ => Ok(false),
    }
}
