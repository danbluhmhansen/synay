use itertools::Itertools;
use pgrx::{
    prelude::*,
    spi::{SpiError, SpiErrorCodes, SpiHeapTupleData},
    JsonB, TimestampWithTimeZone, Uuid, UuidBytes,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PostgresEnum, PartialEq)]
enum EventType {
    Save,
    Drop,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PostgresEnum, PartialEq)]
enum SourceType {
    Game,
}

#[derive(Debug, Deserialize, Serialize, PostgresType)]
struct EventSource {
    id: UuidBytes,
    data: Option<serde_json::Value>,
    event: EventType,
    source: SourceType,
    added: TimestampWithTimeZone,
}

impl EventSource {
    fn new(
        id: UuidBytes,
        data: Option<serde_json::Value>,
        event: EventType,
        source: SourceType,
        added: TimestampWithTimeZone,
    ) -> Self {
        Self {
            id,
            data,
            event,
            source,
            added,
        }
    }
}

impl TryFrom<SpiHeapTupleData<'_>> for EventSource {
    type Error = spi::Error;

    fn try_from(value: SpiHeapTupleData) -> Result<Self, Self::Error> {
        if let (Some(id), Some(event), Some(source), Some(added)) = (
            value["id"].value::<Uuid>()?.map(|id| *id.as_bytes()),
            value["event"].value()?,
            value["source"].value()?,
            value["added"].value()?,
        ) {
            Ok(Self::new(
                id,
                value["data"].value::<JsonB>()?.map(|data| data.0),
                event,
                source,
                added,
            ))
        } else {
            Err(SpiError::SpiError(SpiErrorCodes::TypUnknown))
        }
    }
}

#[pg_extern]
fn event_agg() -> Result<
    TableIterator<
        'static,
        (
            name!(id, Uuid),
            name!(data, JsonB),
            name!(source, Option<SourceType>),
            name!(added, Option<TimestampWithTimeZone>),
            name!(updated, Option<TimestampWithTimeZone>),
        ),
    >,
    spi::Error,
> {
    Ok(TableIterator::new(
        Spi::connect(|client| -> Result<Vec<EventSource>, spi::Error> {
            Ok(client
                .select(
                    "SELECT id, data, event, source, added FROM event_source ORDER BY id, added;",
                    None,
                    None,
                )?
                .filter_map(|row| row.try_into().ok())
                .collect())
        })?
        .into_iter()
        .group_by(|event| event.id)
        .into_iter()
        .filter_map(|(id, events)| {
            let events = events.collect::<Vec<_>>();
            events
                .last()
                .is_some_and(|event| event.event != EventType::Drop)
                .then(|| (id, events))
        })
        .map(|(id, events)| {
            let source = events.first().map(|event| event.source);
            let added = events.first().map(|event| event.added);
            let updated = events.last().map(|event| event.added);
            (
                Uuid::from_bytes(id),
                JsonB(
                    events
                        .into_iter()
                        .filter_map(|event| event.data)
                        .fold(json!({}), |mut acc, data| {
                            json_patch::merge(&mut acc, &data);
                            acc
                        }),
                ),
                source,
                added,
                updated,
            )
        })
        .collect::<Vec<_>>(),
    ))
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::{prelude::*, Uuid};

    #[pg_test]
    fn test_save_event() {
        Spi::run(include_str!("test/save_event.sql")).unwrap();

        assert_eq!(
            Ok(Some("three".to_string())),
            Spi::get_one::<String>("SELECT name FROM game LIMIT 1;")
        );
        assert_eq!(
            Ok(Some("foo".to_string())),
            Spi::get_one::<String>("SELECT description FROM game LIMIT 1;")
        );
    }

    #[pg_test]
    fn test_drop_event() {
        Spi::run(include_str!("test/drop_event.sql")).unwrap();
        let games = Spi::connect(|client| {
            client
                .select("SELECT id FROM game;", None, None)
                .unwrap()
                .map(|row| row["id"].value::<Uuid>())
                .collect::<Vec<_>>()
        });

        assert_eq!(0, games.len());
    }

    #[pg_test]
    fn test_unset_field() {
        Spi::run(include_str!("test/unset_field.sql")).unwrap();

        assert_eq!(
            Ok(Some("three".to_string())),
            Spi::get_one::<String>("SELECT name FROM game LIMIT 1;")
        );
        assert_eq!(
            Ok(None),
            Spi::get_one::<String>("SELECT description FROM game LIMIT 1;")
        );
    }

    #[pg_test]
    fn test_mult_projections() {
        Spi::run(include_str!("test/mult_projections.sql")).unwrap();
        let games = Spi::connect(|client| {
            client
                .select("SELECT id FROM game;", None, None)
                .unwrap()
                .map(|row| row["id"].value::<Uuid>())
                .collect::<Vec<_>>()
        });

        assert_eq!(3, games.len());
    }
}
