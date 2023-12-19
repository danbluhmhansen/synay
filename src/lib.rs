use pgrx::prelude::*;

mod events;

pgrx::pg_module_magic!();

extension_sql_file!("../sql/init.sql", name = "init", requires = [events::EventType]);
extension_sql_file!("../sql/aggs.sql", name = "aggs", requires = ["init", events::event_agg]);

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
