use crate::models::CommandError;
use rusqlite::Connection;

pub struct CommandKnowledgeMigration;

impl CommandKnowledgeMigration {
    pub fn migrate(connection: &Connection) -> Result<(), CommandError> {
        connection
            .execute_batch(
                "
                pragma foreign_keys = on;

                create table if not exists applications (
                    id integer primary key autoincrement,
                    name text not null,
                    path text not null unique,
                    version text,
                    last_scanned_at text not null
                );

                create table if not exists commands (
                    id integer primary key autoincrement,
                    application_id integer not null references applications(id) on delete cascade,
                    name text not null,
                    description text,
                    unique(application_id, name)
                );

                create table if not exists command_options (
                    id integer primary key autoincrement,
                    command_id integer not null references commands(id) on delete cascade,
                    name text not null,
                    short_name text,
                    description text,
                    takes_value integer not null default 0,
                    unique(command_id, name)
                );

                create table if not exists command_arguments (
                    id integer primary key autoincrement,
                    command_id integer not null references commands(id) on delete cascade,
                    name text not null,
                    description text,
                    required integer not null default 0,
                    position integer not null,
                    unique(command_id, name)
                );

                create table if not exists examples (
                    id integer primary key autoincrement,
                    command_id integer not null references commands(id) on delete cascade,
                    title text not null,
                    command_line text not null,
                    description text,
                    unique(command_id, command_line)
                );

                create table if not exists history (
                    id integer primary key autoincrement,
                    command_id integer references commands(id) on delete set null,
                    command_line text not null,
                    working_directory text,
                    shell text,
                    exit_code integer,
                    duration_ms integer,
                    executed_at text not null
                );

                create table if not exists usage_statistics (
                    id integer primary key autoincrement,
                    command_id integer not null references commands(id) on delete cascade,
                    run_count integer not null default 0,
                    last_used_at text,
                    unique(command_id)
                );

                create index if not exists idx_applications_name on applications(name);
                create index if not exists idx_commands_name on commands(name);
                create index if not exists idx_usage_statistics_rank on usage_statistics(run_count desc, last_used_at desc);
                create index if not exists idx_history_command_executed_at on history(command_id, executed_at desc);
                create index if not exists idx_history_executed_at on history(executed_at);
                ",
            )
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        Self::add_column_if_missing(connection, "history", "duration_ms", "integer")?;
        Ok(())
    }

    fn add_column_if_missing(
        connection: &Connection,
        table: &str,
        column: &str,
        definition: &str,
    ) -> Result<(), CommandError> {
        let mut statement = connection
            .prepare(&format!("pragma table_info({table})"))
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;

        if !columns.iter().any(|name| name == column) {
            connection
                .execute(
                    &format!("alter table {table} add column {column} {definition}"),
                    [],
                )
                .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        }

        Ok(())
    }
}
