use itertools::Itertools;
use pgrx::{prelude::*, spi::SpiHeapTupleData, JsonB, TimestampWithTimeZone, Uuid, UuidBytes};
use serde::{Deserialize, Serialize};

pgrx::pg_module_magic!();

#[derive(Deserialize, Serialize, PostgresEnum)]
enum EventType {
    Updated,
    Added,
    Removed,
}

#[derive(Deserialize, Serialize, PostgresType)]
struct GameData {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize, Serialize, PostgresType)]
struct GameEvent {
    id: UuidBytes,
    data: Option<GameData>,
    event: EventType,
    added: TimestampWithTimeZone,
}

impl GameEvent {
    fn new(id: UuidBytes, data: Option<GameData>, event: EventType, added: TimestampWithTimeZone) -> Self {
        Self { id, data, event, added }
    }
}

impl TryFrom<SpiHeapTupleData<'_>> for GameEvent {
    type Error = spi::Error;

    fn try_from(value: SpiHeapTupleData) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value["id"].value::<Uuid>()?.map(|id| *id.as_bytes()).unwrap(),
            value["data"]
                .value::<JsonB>()?
                .and_then(|data| serde_json::from_value(data.0).ok()),
            value["event"].value()?.unwrap(),
            value["added"].value()?.unwrap(),
        ))
    }
}

extension_sql_file!("../sql/init.sql", name = "init", requires = [EventType]);

#[pg_extern]
fn game_agg() -> Result<
    TableIterator<
        'static,
        (
            name!(id, Uuid),
            name!(name, Option<String>),
            name!(description, Option<String>),
        ),
    >,
    spi::Error,
> {
    Ok(TableIterator::new(
        Spi::connect(|client| -> Result<Vec<_>, spi::Error> {
            Ok(client
                .select(
                    "SELECT id, data, event, added FROM game_event WHERE event != 'Removed' ORDER BY added;",
                    None,
                    None,
                )?
                .filter_map(|row| row.try_into().ok())
                .collect::<Vec<GameEvent>>())
        })?
        .into_iter()
        .group_by(|game| game.id)
        .into_iter()
        .map(|(id, group)| {
            let mut name: Option<String> = None;
            let mut description: Option<String> = None;

            for data in group.filter_map(|game| game.data) {
                name = data.name.or(name);
                description = data.description.or(description);
            }

            (Uuid::from_bytes(id), name, description)
        })
        .collect::<Vec<_>>(),
    ))
}

extension_sql_file!("../sql/aggs.sql", name = "aggs", requires = ["init", game_agg]);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::{prelude::*, Uuid};

    #[pg_test]
    fn test_game_agg() {
        Spi::run(include_str!("../sql/tests/game_agg.sql")).unwrap();
        let games = Spi::connect(|client| {
            client
                .select("SELECT id, name, description FROM game;", None, None)
                .unwrap()
                .map(|row| {
                    (
                        row["id"].value::<Uuid>(),
                        row["name"].value::<String>(),
                        row["description"].value::<String>(),
                    )
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(
            Some(&Some("three".to_string())),
            games.first().and_then(|game| game.1.as_ref().ok())
        );

        assert_eq!(
            Some(&Some("foo".to_string())),
            games.first().and_then(|game| game.2.as_ref().ok())
        );
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
