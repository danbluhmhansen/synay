use itertools::Itertools;
use pgrx::{prelude::*, spi::SpiHeapTupleData, JsonB, TimestampWithTimeZone, Uuid, UuidBytes};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, PostgresEnum, PartialEq)]
enum EventType {
    Save,
    Drop,
}

#[derive(Debug, Deserialize, Serialize, PostgresType)]
struct EventSource {
    id: UuidBytes,
    data: Option<serde_json::Value>,
    event: EventType,
    added: TimestampWithTimeZone,
}

impl EventSource {
    fn new(id: UuidBytes, data: Option<serde_json::Value>, event: EventType, added: TimestampWithTimeZone) -> Self {
        Self { id, data, event, added }
    }
}

impl TryFrom<SpiHeapTupleData<'_>> for EventSource {
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
                value["data"].value::<JsonB>()?.map(|data| data.0),
                event,
                added,
            ))
        } else {
            Ok(Self::new(
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                value["data"].value::<JsonB>()?.map(|data| data.0),
                EventType::Save,
                TimestampWithTimeZone::new_unchecked(0, 0, 0, 0, 0, 0.0),
            ))
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
            name!(added, Option<TimestampWithTimeZone>),
            name!(updated, Option<TimestampWithTimeZone>),
        ),
    >,
    spi::Error,
> {
    Ok(TableIterator::new(
        Spi::connect(|client| -> Result<Vec<_>, spi::Error> {
            Ok(client
                .select(
                    "SELECT id, data, event, added FROM event_source ORDER BY id, added;",
                    None,
                    None,
                )?
                .filter_map(|row| row.try_into().ok())
                .collect::<Vec<EventSource>>())
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
            let id = Uuid::from_bytes(id);
            let added = events.first().map(|event| event.added);
            let updated = events.last().map(|event| event.added);
            (
                id,
                JsonB(
                    events
                        .into_iter()
                        .filter_map(|event| event.data)
                        .fold(json!({}), |mut acc, data| {
                            json_patch::merge(&mut acc, &data);
                            acc
                        }),
                ),
                added,
                updated,
            )
        })
        .collect::<Vec<_>>(),
    ))
}

#[pg_extern]
fn save_event(id: Option<Uuid>, data: JsonB) -> Result<(), spi::Error> {
    if let Some(id) = id {
        Spi::run_with_args(
            "INSERT INTO event_source (id, data) VALUES ($1, $2);",
            Some(vec![
                (PgBuiltInOids::UUIDOID.oid(), id.into_datum()),
                (PgBuiltInOids::JSONBOID.oid(), data.into_datum()),
            ]),
        )
    } else {
        Spi::run_with_args(
            "INSERT INTO event_source (data) VALUES ($1);",
            Some(vec![(PgBuiltInOids::JSONBOID.oid(), data.into_datum())]),
        )
    }
}

#[pg_extern]
fn drop_event(id: Uuid) -> Result<(), spi::Error> {
    Spi::run_with_args(
        "INSERT INTO event_source (id, event) VALUES ($1, 'Drop');",
        Some(vec![(PgBuiltInOids::UUIDOID.oid(), id.into_datum())]),
    )
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::{prelude::*, JsonB, Uuid};
    use serde_json::json;

    #[pg_test]
    fn test_save_event() {
        Spi::run(include_str!("test/save_event.sql")).unwrap();
        let data = Spi::connect(|client| {
            client
                .select("SELECT data FROM game;", None, None)
                .unwrap()
                .map(|row| row["data"].value::<JsonB>())
                .collect::<Vec<_>>()
        })
        .into_iter()
        .map(|data| data.unwrap().unwrap().0)
        .collect::<Vec<_>>();

        assert_eq!(Some(&json!({"name": "three", "description": "foo"})), data.first());
    }

    #[pg_test]
    fn test_drop_event() {
        Spi::run(include_str!("test/drop_event.sql")).unwrap();
        let events = Spi::connect(|client| {
            client
                .select("SELECT id, data FROM game;", None, None)
                .unwrap()
                .map(|row| (row["id"].value::<Uuid>(), row["data"].value::<JsonB>()))
                .collect::<Vec<_>>()
        });

        assert_eq!(0, events.len());
    }

    #[pg_test]
    fn test_unset_field() {
        Spi::run(include_str!("test/unset_field.sql")).unwrap();
        let data = Spi::connect(|client| {
            client
                .select("SELECT data FROM game;", None, None)
                .unwrap()
                .map(|row| row["data"].value::<JsonB>())
                .collect::<Vec<_>>()
        })
        .into_iter()
        .map(|data| data.unwrap().unwrap().0)
        .collect::<Vec<_>>();

        assert_eq!(Some(&json!({"name": "three"})), data.first());
    }

    #[pg_test]
    fn test_mult_projections() {
        Spi::run(include_str!("test/mult_projections.sql")).unwrap();
        let events = Spi::connect(|client| {
            client
                .select("SELECT id, data FROM game;", None, None)
                .unwrap()
                .map(|row| (row["id"].value::<Uuid>(), row["data"].value::<JsonB>()))
                .collect::<Vec<_>>()
        });

        assert_eq!(3, events.len());
    }
}
