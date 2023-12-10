use itertools::Itertools;
use pgrx::{prelude::*, spi::SpiHeapTupleData, JsonB, TimestampWithTimeZone, Uuid, UuidBytes};
use serde::{Deserialize, Serialize};

pgrx::pg_module_magic!();

#[derive(Debug, Deserialize, Serialize, PostgresEnum, PartialEq)]
enum EventType {
    Save,
    Drop,
}

#[derive(Debug, Deserialize, Serialize, PostgresType)]
struct GameData {
    name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    description: Option<Option<String>>,
}

#[derive(Debug, Deserialize, Serialize, PostgresType)]
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
        if let Some(((id, event), added)) = value["id"]
            .value::<Uuid>()?
            .map(|id| *id.as_bytes())
            .zip(value["event"].value()?)
            .zip(value["added"].value()?)
        {
            Ok(Self::new(
                id,
                value["data"]
                    .value::<JsonB>()?
                    .and_then(|data| serde_json::from_value(data.0).ok()),
                event,
                added,
            ))
        } else {
            Ok(Self::new(
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                value["data"]
                    .value::<JsonB>()?
                    .and_then(|data| serde_json::from_value(data.0).ok()),
                EventType::Save,
                TimestampWithTimeZone::new_unchecked(0, 0, 0, 0, 0, 0.0),
            ))
        }
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
            name!(description, Option<Option<String>>),
        ),
    >,
    spi::Error,
> {
    Ok(TableIterator::new(
        Spi::connect(|client| -> Result<Vec<_>, spi::Error> {
            Ok(client
                .select(
                    "SELECT id, data, event, added FROM game_event ORDER BY added;",
                    None,
                    None,
                )?
                .filter_map(|row| row.try_into().ok())
                .collect::<Vec<GameEvent>>())
        })?
        .into_iter()
        .group_by(|game| game.id)
        .into_iter()
        .filter_map(|(id, games)| {
            let games = games.collect::<Vec<_>>();
            games
                .last()
                .is_some_and(|game| game.event != EventType::Drop)
                .then(|| (id, games))
        })
        .map(|(id, group)| {
            group.into_iter().filter_map(|game| game.data).fold(
                (Uuid::from_bytes(id), None::<String>, None::<Option<String>>),
                |mut acc, GameData { name, description }| {
                    acc.1 = name.or(acc.1);
                    acc.2 = description.or(acc.2);
                    acc
                },
            )
        })
        .collect::<Vec<_>>(),
    ))
}

extension_sql_file!("../sql/aggs.sql", name = "aggs", requires = ["init", game_agg]);

#[pg_extern]
fn save_game(id: Option<Uuid>, data: JsonB) -> Result<(), spi::Error> {
    if let Some(id) = id {
        Spi::run_with_args(
            "INSERT INTO game_event (id, data) VALUES ($1, $2);",
            Some(vec![
                (PgBuiltInOids::UUIDOID.oid(), id.into_datum()),
                (PgBuiltInOids::JSONBOID.oid(), data.into_datum()),
            ]),
        )
    } else {
        Spi::run_with_args(
            "INSERT INTO game_event (data) VALUES ($1);",
            Some(vec![(PgBuiltInOids::JSONBOID.oid(), data.into_datum())]),
        )
    }
}

#[pg_extern]
fn drop_game(id: Uuid) -> Result<(), spi::Error> {
    Spi::run_with_args(
        "INSERT INTO game_event (id, event) VALUES ($1, 'Drop');",
        Some(vec![(PgBuiltInOids::UUIDOID.oid(), id.into_datum())]),
    )
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::{prelude::*, Uuid};

    #[pg_test]
    fn test_game_agg() {
        Spi::run(include_str!("test/game_agg.sql")).unwrap();
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

    #[pg_test]
    fn test_drop_game() {
        Spi::run(include_str!("test/drop_game.sql")).unwrap();
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

        assert_eq!(0, games.len());
    }

    #[pg_test]
    fn test_unset_field() {
        Spi::run(include_str!("test/unset_field.sql")).unwrap();
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

        assert_eq!(Some(&None), games.first().and_then(|game| game.2.as_ref().ok()));
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
