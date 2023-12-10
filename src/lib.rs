use pgrx::{prelude::*, PostgresEnum};
use serde::{Deserialize, Serialize};

mod game;

pgrx::pg_module_magic!();

#[derive(Debug, Deserialize, Serialize, PostgresEnum, PartialEq)]
enum EventType {
    Save,
    Drop,
}

extension_sql_file!("../sql/init.sql", name = "init", requires = [EventType]);

extension_sql_file!("../sql/aggs.sql", name = "aggs", requires = ["init", game::game_agg]);

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
