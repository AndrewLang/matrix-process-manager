use crate::command_knowledge::models::*;
use crate::models::CommandError;
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

pub type SharedConnection = Arc<Mutex<Connection>>;

pub struct ApplicationRepository {
    connection: SharedConnection,
}

impl ApplicationRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn create(&self, record: &NewApplicationRecord) -> Result<ApplicationRecord, CommandError> {
        let connection = self.lock()?;
        connection.execute(
            "insert into applications (name, path, version, last_scanned_at) values (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![record.name, record.path, record.version],
        ).map_err(Self::error)?;
        Self::find_by_id_with_connection(&connection, connection.last_insert_rowid())
    }

    pub fn upsert_scanned(
        &self,
        record: &NewApplicationRecord,
    ) -> Result<(ApplicationRecord, bool), CommandError> {
        let connection = self.lock()?;
        let existing = Self::find_by_path_with_connection(&connection, &record.path)?;
        connection.execute(
            "insert into applications (name, path, version, last_scanned_at) values (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) on conflict(path) do update set name = excluded.name, version = excluded.version, last_scanned_at = excluded.last_scanned_at",
            params![record.name, record.path, record.version],
        ).map_err(Self::error)?;
        Ok((
            Self::find_by_path_with_connection(&connection, &record.path)?
                .ok_or_else(|| CommandError::terminal_failed("application was not saved"))?,
            existing.is_none(),
        ))
    }

    pub fn get(&self, id: i64) -> Result<Option<ApplicationRecord>, CommandError> {
        let connection = self.lock()?;
        Self::find_by_id_optional_with_connection(&connection, id)
    }

    pub fn list(&self) -> Result<Vec<ApplicationRecord>, CommandError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "select id, name, path, version, last_scanned_at from applications order by name",
            )
            .map_err(Self::error)?;
        let rows = statement
            .query_map([], Self::map_application)
            .map_err(Self::error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::error)
    }

    pub fn find_by_name(&self, name: &str) -> Result<Option<ApplicationRecord>, CommandError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "select id, name, path, version, last_scanned_at from applications where lower(name) = lower(?1) order by id limit 1",
                params![name],
                Self::map_application,
            )
            .optional()
            .map_err(Self::error)
    }

    pub fn upsert_manual(&self, name: &str) -> Result<ApplicationRecord, CommandError> {
        let record = NewApplicationRecord {
            name: name.to_string(),
            path: format!("manual:{name}"),
            version: None,
        };
        self.upsert_scanned(&record)
            .map(|(application, _)| application)
    }

    pub fn update(&self, record: &ApplicationRecord) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute(
            "update applications set name = ?1, path = ?2, version = ?3, last_scanned_at = ?4 where id = ?5",
            params![record.name, record.path, record.version, record.last_scanned_at, record.id],
        ).map_err(Self::error)?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection
            .execute("delete from applications where id = ?1", params![id])
            .map_err(Self::error)?;
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CommandError> {
        self.connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("command knowledge database is unavailable"))
    }

    fn find_by_id_with_connection(
        connection: &Connection,
        id: i64,
    ) -> Result<ApplicationRecord, CommandError> {
        Self::find_by_id_optional_with_connection(connection, id)?
            .ok_or_else(|| CommandError::terminal_failed("application was not found"))
    }

    fn find_by_id_optional_with_connection(
        connection: &Connection,
        id: i64,
    ) -> Result<Option<ApplicationRecord>, CommandError> {
        connection
            .query_row(
                "select id, name, path, version, last_scanned_at from applications where id = ?1",
                params![id],
                Self::map_application,
            )
            .optional()
            .map_err(Self::error)
    }

    fn find_by_path_with_connection(
        connection: &Connection,
        path: &str,
    ) -> Result<Option<ApplicationRecord>, CommandError> {
        connection
            .query_row(
                "select id, name, path, version, last_scanned_at from applications where path = ?1",
                params![path],
                Self::map_application,
            )
            .optional()
            .map_err(Self::error)
    }

    fn map_application(row: &rusqlite::Row<'_>) -> rusqlite::Result<ApplicationRecord> {
        Ok(ApplicationRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            version: row.get(3)?,
            last_scanned_at: row.get(4)?,
        })
    }

    fn error(error: rusqlite::Error) -> CommandError {
        CommandError::terminal_failed(error.to_string())
    }
}

pub struct CommandRepository {
    connection: SharedConnection,
}

impl CommandRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn create(&self, record: &NewCommandRecord) -> Result<CommandRecord, CommandError> {
        let connection = self.lock()?;
        connection
            .execute(
                "insert into commands (application_id, name, description) values (?1, ?2, ?3)",
                params![record.application_id, record.name, record.description],
            )
            .map_err(Self::error)?;
        self.get_with_connection(&connection, connection.last_insert_rowid())?
            .ok_or_else(|| CommandError::terminal_failed("command was not saved"))
    }

    pub fn upsert(&self, record: &NewCommandRecord) -> Result<CommandRecord, CommandError> {
        let connection = self.lock()?;
        connection.execute("insert into commands (application_id, name, description) values (?1, ?2, ?3) on conflict(application_id, name) do update set description = coalesce(excluded.description, commands.description)", params![record.application_id, record.name, record.description]).map_err(Self::error)?;
        self.find_by_application_and_name_with_connection(
            &connection,
            record.application_id,
            &record.name,
        )?
        .ok_or_else(|| CommandError::terminal_failed("command was not saved"))
    }

    pub fn get(&self, id: i64) -> Result<Option<CommandRecord>, CommandError> {
        let connection = self.lock()?;
        self.get_with_connection(&connection, id)
    }

    pub fn list_by_application(
        &self,
        application_id: i64,
    ) -> Result<Vec<CommandRecord>, CommandError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("select id, application_id, name, description from commands where application_id = ?1 order by name").map_err(Self::error)?;
        let rows = statement
            .query_map(params![application_id], Self::map_command)
            .map_err(Self::error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::error)
    }

    pub fn find_by_name(&self, name: &str) -> Result<Option<CommandRecord>, CommandError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "select id, application_id, name, description from commands where name = ?1",
                params![name],
                Self::map_command,
            )
            .optional()
            .map_err(Self::error)
    }

    pub fn autocomplete(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<CommandAutocompleteSuggestion>, CommandError> {
        let normalized_query = query.trim().to_ascii_lowercase();
        if normalized_query.is_empty() {
            return Ok(Vec::new());
        }

        let connection = self.lock()?;
        let mut statement = connection.prepare("select c.id, c.name, c.description, coalesce(us.run_count, 0), us.last_used_at from commands c left join usage_statistics us on us.command_id = c.id").map_err(Self::error)?;
        let rows = statement
            .query_map([], |row| {
                Ok(CommandAutocompleteCandidate {
                    command_id: row.get(0)?,
                    command_line: row.get(1)?,
                    description: row.get(2)?,
                    usage_count: row.get(3)?,
                    usage_last_used_at: row.get(4)?,
                })
            })
            .map_err(Self::error)?;

        let mut suggestions = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(Self::error)?
            .into_iter()
            .filter_map(|candidate| Self::score_candidate(candidate, &normalized_query))
            .collect::<Vec<_>>();

        suggestions.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| right.usage_count.cmp(&left.usage_count))
                .then_with(|| right.last_used_at.cmp(&left.last_used_at))
                .then_with(|| left.command_line.cmp(&right.command_line))
        });
        suggestions.truncate(limit.clamp(1, 20));
        for suggestion in &mut suggestions {
            Self::enrich_suggestion(&connection, suggestion)?;
        }
        Ok(suggestions)
    }

    pub fn update(&self, record: &CommandRecord) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute("update commands set application_id = ?1, name = ?2, description = ?3 where id = ?4", params![record.application_id, record.name, record.description, record.id]).map_err(Self::error)?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection
            .execute("delete from commands where id = ?1", params![id])
            .map_err(Self::error)?;
        Ok(())
    }

    fn get_with_connection(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<Option<CommandRecord>, CommandError> {
        connection
            .query_row(
                "select id, application_id, name, description from commands where id = ?1",
                params![id],
                Self::map_command,
            )
            .optional()
            .map_err(Self::error)
    }

    fn find_by_application_and_name_with_connection(
        &self,
        connection: &Connection,
        application_id: i64,
        name: &str,
    ) -> Result<Option<CommandRecord>, CommandError> {
        connection
            .query_row(
                "select id, application_id, name, description from commands where application_id = ?1 and name = ?2",
                params![application_id, name],
                Self::map_command,
            )
            .optional()
            .map_err(Self::error)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CommandError> {
        self.connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("command knowledge database is unavailable"))
    }

    fn map_command(row: &rusqlite::Row<'_>) -> rusqlite::Result<CommandRecord> {
        Ok(CommandRecord {
            id: row.get(0)?,
            application_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
        })
    }

    fn error(error: rusqlite::Error) -> CommandError {
        CommandError::terminal_failed(error.to_string())
    }

    fn score_candidate(
        candidate: CommandAutocompleteCandidate,
        query: &str,
    ) -> Option<CommandAutocompleteSuggestion> {
        let command_line = candidate.command_line.to_ascii_lowercase();
        let label = Self::label_for_query(&candidate.command_line, query);
        let label_lower = label.to_ascii_lowercase();
        let prefix_score = if command_line.starts_with(query) {
            140
        } else if label_lower.starts_with(query.split_whitespace().last().unwrap_or(query)) {
            120
        } else if command_line.contains(query) {
            80
        } else {
            Self::fuzzy_score(&command_line, query)?
        };
        let usage_score = candidate.usage_count.min(100) * 3;
        let recent_score = candidate.last_used_at().map(|_| 25).unwrap_or_default();

        let last_used_at = candidate.last_used_at();
        let frequently_used = candidate.usage_count >= 5;
        let recently_used = last_used_at.is_some();

        Some(CommandAutocompleteSuggestion {
            command_id: candidate.command_id,
            command_line: candidate.command_line,
            label,
            icon: "bi-terminal".to_string(),
            description: candidate.description,
            examples: Vec::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            usage_count: candidate.usage_count,
            score: prefix_score + usage_score + recent_score,
            last_used_at,
            frequently_used,
            recently_used,
        })
    }

    fn enrich_suggestion(
        connection: &Connection,
        suggestion: &mut CommandAutocompleteSuggestion,
    ) -> Result<(), CommandError> {
        suggestion.examples = Self::autocomplete_examples(connection, suggestion.command_id)?;
        suggestion.arguments = Self::autocomplete_arguments(connection, suggestion.command_id)?;
        suggestion.options = Self::autocomplete_options(connection, suggestion.command_id)?;
        suggestion.icon = Self::icon_for_command(&suggestion.command_line);
        Ok(())
    }

    fn autocomplete_examples(
        connection: &Connection,
        command_id: i64,
    ) -> Result<Vec<CommandAutocompleteExample>, CommandError> {
        let mut statement = connection.prepare("select title, command_line, description from examples where command_id = ?1 order by title limit 3").map_err(Self::error)?;
        let rows = statement
            .query_map(params![command_id], |row| {
                Ok(CommandAutocompleteExample {
                    title: row.get(0)?,
                    command_line: row.get(1)?,
                    description: row.get(2)?,
                })
            })
            .map_err(Self::error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::error)
    }

    fn autocomplete_arguments(
        connection: &Connection,
        command_id: i64,
    ) -> Result<Vec<CommandAutocompleteArgument>, CommandError> {
        let mut statement = connection.prepare("select name, description, required from command_arguments where command_id = ?1 order by position limit 6").map_err(Self::error)?;
        let rows = statement
            .query_map(params![command_id], |row| {
                Ok(CommandAutocompleteArgument {
                    name: row.get(0)?,
                    description: row.get(1)?,
                    required: row.get(2)?,
                })
            })
            .map_err(Self::error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::error)
    }

    fn autocomplete_options(
        connection: &Connection,
        command_id: i64,
    ) -> Result<Vec<CommandAutocompleteOption>, CommandError> {
        let mut statement = connection.prepare("select name, short_name, description, takes_value from command_options where command_id = ?1 order by name limit 8").map_err(Self::error)?;
        let rows = statement
            .query_map(params![command_id], |row| {
                Ok(CommandAutocompleteOption {
                    name: row.get(0)?,
                    short_name: row.get(1)?,
                    description: row.get(2)?,
                    takes_value: row.get(3)?,
                })
            })
            .map_err(Self::error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::error)
    }

    fn icon_for_command(command_line: &str) -> String {
        match command_line
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "docker" => "bi-box-seam".to_string(),
            "git" => "bi-git".to_string(),
            "cargo" => "bi-box".to_string(),
            "pnpm" | "npm" | "node" => "bi-node-plus".to_string(),
            _ => "bi-terminal".to_string(),
        }
    }

    fn label_for_query(command_line: &str, query: &str) -> String {
        let query_parts = query.split_whitespace().collect::<Vec<_>>();
        let command_parts = command_line.split_whitespace().collect::<Vec<_>>();
        if query_parts.len() > 1 && command_parts.len() >= query_parts.len() {
            command_parts[query_parts.len() - 1].to_string()
        } else {
            command_line.to_string()
        }
    }

    fn fuzzy_score(candidate: &str, query: &str) -> Option<i64> {
        let mut score = 0;
        let mut last_index = 0;
        for character in query.chars().filter(|character| !character.is_whitespace()) {
            let remaining = &candidate[last_index..];
            let Some(index) = remaining.find(character) else {
                return None;
            };
            score += if index == 0 { 6 } else { 2 };
            last_index += index + character.len_utf8();
        }
        Some(score)
    }
}

struct CommandAutocompleteCandidate {
    command_id: i64,
    command_line: String,
    description: Option<String>,
    usage_count: i64,
    usage_last_used_at: Option<String>,
}

impl CommandAutocompleteCandidate {
    fn last_used_at(&self) -> Option<String> {
        self.usage_last_used_at.clone()
    }
}

pub struct CommandOptionRepository {
    connection: SharedConnection,
}

impl CommandOptionRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn create(
        &self,
        record: &NewCommandOptionRecord,
    ) -> Result<CommandOptionRecord, CommandError> {
        let connection = self.lock()?;
        connection.execute("insert into command_options (command_id, name, short_name, description, takes_value) values (?1, ?2, ?3, ?4, ?5)", params![record.command_id, record.name, record.short_name, record.description, record.takes_value]).map_err(Self::error)?;
        self.get_with_connection(&connection, connection.last_insert_rowid())?
            .ok_or_else(|| CommandError::terminal_failed("command option was not saved"))
    }

    pub fn upsert(&self, record: &NewCommandOptionRecord) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute("insert into command_options (command_id, name, short_name, description, takes_value) values (?1, ?2, ?3, ?4, ?5) on conflict(command_id, name) do update set short_name = coalesce(excluded.short_name, command_options.short_name), description = coalesce(excluded.description, command_options.description), takes_value = excluded.takes_value", params![record.command_id, record.name, record.short_name, record.description, record.takes_value]).map_err(Self::error)?;
        Ok(())
    }

    pub fn get(&self, id: i64) -> Result<Option<CommandOptionRecord>, CommandError> {
        let connection = self.lock()?;
        self.get_with_connection(&connection, id)
    }

    pub fn list_by_command(
        &self,
        command_id: i64,
    ) -> Result<Vec<CommandOptionRecord>, CommandError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("select id, command_id, name, short_name, description, takes_value from command_options where command_id = ?1 order by name").map_err(Self::error)?;
        let rows = statement
            .query_map(params![command_id], Self::map_option)
            .map_err(Self::error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::error)
    }

    pub fn update(&self, record: &CommandOptionRecord) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute("update command_options set command_id = ?1, name = ?2, short_name = ?3, description = ?4, takes_value = ?5 where id = ?6", params![record.command_id, record.name, record.short_name, record.description, record.takes_value, record.id]).map_err(Self::error)?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), CommandError> {
        self.delete_from("command_options", id)
    }

    fn delete_from(&self, table: &str, id: i64) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection
            .execute(&format!("delete from {table} where id = ?1"), params![id])
            .map_err(Self::error)?;
        Ok(())
    }

    fn get_with_connection(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<Option<CommandOptionRecord>, CommandError> {
        connection.query_row("select id, command_id, name, short_name, description, takes_value from command_options where id = ?1", params![id], Self::map_option).optional().map_err(Self::error)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CommandError> {
        self.connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("command knowledge database is unavailable"))
    }

    fn map_option(row: &rusqlite::Row<'_>) -> rusqlite::Result<CommandOptionRecord> {
        Ok(CommandOptionRecord {
            id: row.get(0)?,
            command_id: row.get(1)?,
            name: row.get(2)?,
            short_name: row.get(3)?,
            description: row.get(4)?,
            takes_value: row.get(5)?,
        })
    }

    fn error(error: rusqlite::Error) -> CommandError {
        CommandError::terminal_failed(error.to_string())
    }
}

pub struct CommandArgumentRepository {
    connection: SharedConnection,
}

impl CommandArgumentRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn create(
        &self,
        record: &NewCommandArgumentRecord,
    ) -> Result<CommandArgumentRecord, CommandError> {
        let connection = self.lock()?;
        connection.execute("insert into command_arguments (command_id, name, description, required, position) values (?1, ?2, ?3, ?4, ?5)", params![record.command_id, record.name, record.description, record.required, record.position]).map_err(Self::error)?;
        self.get_with_connection(&connection, connection.last_insert_rowid())?
            .ok_or_else(|| CommandError::terminal_failed("command argument was not saved"))
    }

    pub fn get(&self, id: i64) -> Result<Option<CommandArgumentRecord>, CommandError> {
        let connection = self.lock()?;
        self.get_with_connection(&connection, id)
    }

    pub fn list_by_command(
        &self,
        command_id: i64,
    ) -> Result<Vec<CommandArgumentRecord>, CommandError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("select id, command_id, name, description, required, position from command_arguments where command_id = ?1 order by position").map_err(Self::error)?;
        let rows = statement
            .query_map(params![command_id], Self::map_argument)
            .map_err(Self::error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::error)
    }

    pub fn update(&self, record: &CommandArgumentRecord) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute("update command_arguments set command_id = ?1, name = ?2, description = ?3, required = ?4, position = ?5 where id = ?6", params![record.command_id, record.name, record.description, record.required, record.position, record.id]).map_err(Self::error)?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection
            .execute("delete from command_arguments where id = ?1", params![id])
            .map_err(Self::error)?;
        Ok(())
    }

    fn get_with_connection(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<Option<CommandArgumentRecord>, CommandError> {
        connection.query_row("select id, command_id, name, description, required, position from command_arguments where id = ?1", params![id], Self::map_argument).optional().map_err(Self::error)
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CommandError> {
        self.connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("command knowledge database is unavailable"))
    }
    fn map_argument(row: &rusqlite::Row<'_>) -> rusqlite::Result<CommandArgumentRecord> {
        Ok(CommandArgumentRecord {
            id: row.get(0)?,
            command_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            required: row.get(4)?,
            position: row.get(5)?,
        })
    }
    fn error(error: rusqlite::Error) -> CommandError {
        CommandError::terminal_failed(error.to_string())
    }
}

pub struct ExampleRepository {
    connection: SharedConnection,
}

impl ExampleRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }
    pub fn create(&self, record: &NewExampleRecord) -> Result<ExampleRecord, CommandError> {
        let connection = self.lock()?;
        connection.execute("insert into examples (command_id, title, command_line, description) values (?1, ?2, ?3, ?4)", params![record.command_id, record.title, record.command_line, record.description]).map_err(Self::error)?;
        self.get_with_connection(&connection, connection.last_insert_rowid())?
            .ok_or_else(|| CommandError::terminal_failed("example was not saved"))
    }
    pub fn upsert(&self, record: &NewExampleRecord) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute("insert into examples (command_id, title, command_line, description) values (?1, ?2, ?3, ?4) on conflict(command_id, command_line) do update set title = excluded.title, description = coalesce(excluded.description, examples.description)", params![record.command_id, record.title, record.command_line, record.description]).map_err(Self::error)?;
        Ok(())
    }
    pub fn get(&self, id: i64) -> Result<Option<ExampleRecord>, CommandError> {
        let connection = self.lock()?;
        self.get_with_connection(&connection, id)
    }
    pub fn list_by_command(&self, command_id: i64) -> Result<Vec<ExampleRecord>, CommandError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("select id, command_id, title, command_line, description from examples where command_id = ?1 order by title").map_err(Self::error)?;
        let rows = statement
            .query_map(params![command_id], Self::map_example)
            .map_err(Self::error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::error)
    }
    pub fn update(&self, record: &ExampleRecord) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute("update examples set command_id = ?1, title = ?2, command_line = ?3, description = ?4 where id = ?5", params![record.command_id, record.title, record.command_line, record.description, record.id]).map_err(Self::error)?;
        Ok(())
    }
    pub fn delete(&self, id: i64) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection
            .execute("delete from examples where id = ?1", params![id])
            .map_err(Self::error)?;
        Ok(())
    }
    fn get_with_connection(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<Option<ExampleRecord>, CommandError> {
        connection.query_row("select id, command_id, title, command_line, description from examples where id = ?1", params![id], Self::map_example).optional().map_err(Self::error)
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CommandError> {
        self.connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("command knowledge database is unavailable"))
    }
    fn map_example(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExampleRecord> {
        Ok(ExampleRecord {
            id: row.get(0)?,
            command_id: row.get(1)?,
            title: row.get(2)?,
            command_line: row.get(3)?,
            description: row.get(4)?,
        })
    }
    fn error(error: rusqlite::Error) -> CommandError {
        CommandError::terminal_failed(error.to_string())
    }
}

pub struct HistoryRepository {
    connection: SharedConnection,
}

impl HistoryRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }
    pub fn create(&self, record: &NewHistoryRecord) -> Result<HistoryRecord, CommandError> {
        let connection = self.lock()?;
        connection.execute("insert into history (command_id, command_line, working_directory, shell, exit_code, duration_ms, executed_at) values (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))", params![record.command_id, record.command_line, record.working_directory, record.shell, record.exit_code, record.duration_ms]).map_err(Self::error)?;
        self.get_with_connection(&connection, connection.last_insert_rowid())?
            .ok_or_else(|| CommandError::terminal_failed("history was not saved"))
    }
    pub fn get(&self, id: i64) -> Result<Option<HistoryRecord>, CommandError> {
        let connection = self.lock()?;
        self.get_with_connection(&connection, id)
    }
    pub fn list_recent(&self, limit: i64) -> Result<Vec<HistoryRecord>, CommandError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("select id, command_id, command_line, working_directory, shell, exit_code, duration_ms, executed_at from history order by executed_at desc limit ?1").map_err(Self::error)?;
        let rows = statement
            .query_map(params![limit], Self::map_history)
            .map_err(Self::error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::error)
    }
    pub fn update(&self, record: &HistoryRecord) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute("update history set command_id = ?1, command_line = ?2, working_directory = ?3, shell = ?4, exit_code = ?5, duration_ms = ?6, executed_at = ?7 where id = ?8", params![record.command_id, record.command_line, record.working_directory, record.shell, record.exit_code, record.duration_ms, record.executed_at, record.id]).map_err(Self::error)?;
        Ok(())
    }
    pub fn complete(
        &self,
        id: i64,
        exit_code: Option<i64>,
        duration_ms: Option<i64>,
    ) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection
            .execute(
                "update history set exit_code = ?1, duration_ms = ?2 where id = ?3",
                params![exit_code, duration_ms, id],
            )
            .map_err(Self::error)?;
        Ok(())
    }
    pub fn delete(&self, id: i64) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection
            .execute("delete from history where id = ?1", params![id])
            .map_err(Self::error)?;
        Ok(())
    }
    fn get_with_connection(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<Option<HistoryRecord>, CommandError> {
        connection.query_row("select id, command_id, command_line, working_directory, shell, exit_code, duration_ms, executed_at from history where id = ?1", params![id], Self::map_history).optional().map_err(Self::error)
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CommandError> {
        self.connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("command knowledge database is unavailable"))
    }
    fn map_history(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryRecord> {
        Ok(HistoryRecord {
            id: row.get(0)?,
            command_id: row.get(1)?,
            command_line: row.get(2)?,
            working_directory: row.get(3)?,
            shell: row.get(4)?,
            exit_code: row.get(5)?,
            duration_ms: row.get(6)?,
            executed_at: row.get(7)?,
        })
    }
    fn error(error: rusqlite::Error) -> CommandError {
        CommandError::terminal_failed(error.to_string())
    }
}

pub struct UsageStatisticRepository {
    connection: SharedConnection,
}

impl UsageStatisticRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }
    pub fn create(
        &self,
        record: &NewUsageStatisticRecord,
    ) -> Result<UsageStatisticRecord, CommandError> {
        let connection = self.lock()?;
        connection.execute("insert into usage_statistics (command_id, run_count, last_used_at) values (?1, ?2, ?3)", params![record.command_id, record.run_count, record.last_used_at]).map_err(Self::error)?;
        self.get_with_connection(&connection, connection.last_insert_rowid())?
            .ok_or_else(|| CommandError::terminal_failed("usage statistic was not saved"))
    }
    pub fn get(&self, id: i64) -> Result<Option<UsageStatisticRecord>, CommandError> {
        let connection = self.lock()?;
        self.get_with_connection(&connection, id)
    }
    pub fn increment(&self, command_id: i64) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute("insert into usage_statistics (command_id, run_count, last_used_at) values (?1, 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) on conflict(command_id) do update set run_count = run_count + 1, last_used_at = excluded.last_used_at", params![command_id]).map_err(Self::error)?;
        Ok(())
    }
    pub fn update(&self, record: &UsageStatisticRecord) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection.execute("update usage_statistics set command_id = ?1, run_count = ?2, last_used_at = ?3 where id = ?4", params![record.command_id, record.run_count, record.last_used_at, record.id]).map_err(Self::error)?;
        Ok(())
    }
    pub fn delete(&self, id: i64) -> Result<(), CommandError> {
        let connection = self.lock()?;
        connection
            .execute("delete from usage_statistics where id = ?1", params![id])
            .map_err(Self::error)?;
        Ok(())
    }
    fn get_with_connection(
        &self,
        connection: &Connection,
        id: i64,
    ) -> Result<Option<UsageStatisticRecord>, CommandError> {
        connection.query_row("select id, command_id, run_count, last_used_at from usage_statistics where id = ?1", params![id], Self::map_usage).optional().map_err(Self::error)
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CommandError> {
        self.connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("command knowledge database is unavailable"))
    }
    fn map_usage(row: &rusqlite::Row<'_>) -> rusqlite::Result<UsageStatisticRecord> {
        Ok(UsageStatisticRecord {
            id: row.get(0)?,
            command_id: row.get(1)?,
            run_count: row.get(2)?,
            last_used_at: row.get(3)?,
        })
    }
    fn error(error: rusqlite::Error) -> CommandError {
        CommandError::terminal_failed(error.to_string())
    }
}
