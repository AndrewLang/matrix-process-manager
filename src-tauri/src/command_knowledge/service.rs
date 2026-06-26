use crate::command_knowledge::database::CommandKnowledgeDatabase;
use crate::command_knowledge::indexer::CommandIndexer;
use crate::command_knowledge::migration::CommandKnowledgeMigration;
use crate::command_knowledge::models::{
    CommandAutocompleteRequest, CommandAutocompleteSuggestion, CommandIndexResult, CommandRecord,
    FinishCommandExecutionRequest, InstalledApplicationScanResult, NewCommandRecord,
    NewHistoryRecord, StartCommandExecutionRequest, StartCommandExecutionResponse,
};
use crate::command_knowledge::repositories::{
    ApplicationRepository, CommandArgumentRepository, CommandOptionRepository, CommandRepository,
    ExampleRepository, HistoryRepository, SharedConnection, UsageStatisticRepository,
};
use crate::command_knowledge::scanner::{ApplicationScanner, InstalledApplicationScanner};
use crate::models::CommandError;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct CommandKnowledgeService {
    pub applications: ApplicationRepository,
    pub commands: CommandRepository,
    pub command_options: CommandOptionRepository,
    pub command_arguments: CommandArgumentRepository,
    pub examples: ExampleRepository,
    pub history: HistoryRepository,
    pub usage_statistics: UsageStatisticRepository,
    application_scanners: Vec<Box<dyn ApplicationScanner>>,
    indexed_applications: Mutex<HashSet<String>>,
}

impl CommandKnowledgeService {
    pub fn new() -> Result<Self, CommandError> {
        let connection = CommandKnowledgeDatabase::open()?;
        CommandKnowledgeMigration::migrate(&connection)?;
        let connection: SharedConnection = Arc::new(Mutex::new(connection));

        Ok(Self {
            applications: ApplicationRepository::new(connection.clone()),
            commands: CommandRepository::new(connection.clone()),
            command_options: CommandOptionRepository::new(connection.clone()),
            command_arguments: CommandArgumentRepository::new(connection.clone()),
            examples: ExampleRepository::new(connection.clone()),
            history: HistoryRepository::new(connection.clone()),
            usage_statistics: UsageStatisticRepository::new(connection),
            application_scanners: vec![Box::new(InstalledApplicationScanner::new())],
            indexed_applications: Mutex::new(HashSet::new()),
        })
    }

    pub fn scan_installed_applications(
        &self,
    ) -> Result<InstalledApplicationScanResult, CommandError> {
        let mut scanned = 0;
        let mut inserted = 0;
        let mut updated = 0;

        for scanner in &self.application_scanners {
            let applications = scanner.scan()?;
            scanned += applications.len();

            for application in applications {
                let (_, was_inserted) = self.applications.upsert_scanned(&application)?;
                if was_inserted {
                    inserted += 1;
                } else {
                    updated += 1;
                }
            }
        }

        Ok(InstalledApplicationScanResult {
            scanned,
            inserted,
            updated,
        })
    }

    pub fn index_commands(&self) -> Result<CommandIndexResult, CommandError> {
        CommandIndexer::new(
            &self.applications,
            &self.commands,
            &self.command_options,
            &self.examples,
        )
        .index()
    }

    pub fn autocomplete_commands(
        &self,
        request: CommandAutocompleteRequest,
    ) -> Result<Vec<CommandAutocompleteSuggestion>, CommandError> {
        let limit = request.limit.unwrap_or(8);
        let suggestions = self.commands.autocomplete(&request.query, limit)?;
        if !self.should_index_for_autocomplete(&request.query, &suggestions)? {
            return Ok(suggestions);
        }

        self.index_query_application(&request.query)?;
        self.commands.autocomplete(&request.query, limit)
    }

    pub fn start_command_execution(
        &self,
        request: StartCommandExecutionRequest,
    ) -> Result<StartCommandExecutionResponse, CommandError> {
        let command = self.ensure_command(&request.command_line)?;

        let history = self.history.create(&NewHistoryRecord {
            command_id: Some(command.id),
            command_line: request.command_line,
            working_directory: request.working_directory,
            shell: request.shell,
            exit_code: None,
            duration_ms: None,
        })?;

        self.usage_statistics.increment(command.id)?;

        Ok(StartCommandExecutionResponse {
            history_id: history.id,
        })
    }

    pub fn finish_command_execution(
        &self,
        request: FinishCommandExecutionRequest,
    ) -> Result<(), CommandError> {
        self.history
            .complete(request.history_id, request.exit_code, request.duration_ms)
    }

    fn ensure_command(&self, command_line: &str) -> Result<CommandRecord, CommandError> {
        if let Some(command) = self.commands.find_by_name(command_line)? {
            return Ok(command);
        }

        let application_name = CommandLineParser::new(command_line).application_name();
        let application = match self.applications.find_by_name(&application_name)? {
            Some(application) => application,
            None => self.applications.upsert_manual(&application_name)?,
        };

        self.commands.upsert(&NewCommandRecord {
            application_id: application.id,
            name: command_line.trim().to_string(),
            description: None,
        })
    }

    fn index_query_application(&self, query: &str) -> Result<(), CommandError> {
        let application_name = CommandLineParser::new(query).application_name();
        if application_name == "unknown" || application_name.len() < 2 {
            return Ok(());
        }

        let scanner = InstalledApplicationScanner::new();
        let application = match scanner.find_by_name(&application_name)? {
            Some(record) => self.applications.upsert_scanned(&record)?.0,
            None => match self.applications.find_by_name(&application_name)? {
                Some(application) if !application.path.starts_with("manual:") => application,
                _ => return Ok(()),
            },
        };

        CommandIndexer::new(
            &self.applications,
            &self.commands,
            &self.command_options,
            &self.examples,
        )
        .index_application_overview(&application)?;

        Ok(())
    }

    fn should_index_for_autocomplete(
        &self,
        query: &str,
        suggestions: &[CommandAutocompleteSuggestion],
    ) -> Result<bool, CommandError> {
        let application_name = CommandLineParser::new(query)
            .application_name()
            .to_ascii_lowercase();
        if application_name == "unknown" || application_name.len() < 2 {
            return Ok(false);
        }

        if suggestions.iter().any(|suggestion| {
            suggestion
                .command_line
                .to_ascii_lowercase()
                .starts_with(&format!("{application_name} "))
        }) {
            return Ok(false);
        }

        let mut indexed = self
            .indexed_applications
            .lock()
            .map_err(|_| CommandError::terminal_failed("command knowledge index is unavailable"))?;
        Ok(indexed.insert(application_name))
    }
}

struct CommandLineParser<'a> {
    command_line: &'a str,
}

impl<'a> CommandLineParser<'a> {
    fn new(command_line: &'a str) -> Self {
        Self { command_line }
    }

    fn application_name(&self) -> String {
        let token = self.first_token();
        let name = std::path::Path::new(&token)
            .file_stem()
            .or_else(|| std::path::Path::new(&token).file_name())
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or(token);

        if name.trim().is_empty() {
            "unknown".to_string()
        } else {
            name
        }
    }

    fn first_token(&self) -> String {
        let mut token = String::new();
        let mut quote: Option<char> = None;
        let mut escaped = false;

        for character in self.command_line.trim().chars() {
            if escaped {
                token.push(character);
                escaped = false;
                continue;
            }

            if character == '\\' {
                escaped = true;
                continue;
            }

            if let Some(current_quote) = quote {
                if character == current_quote {
                    quote = None;
                } else {
                    token.push(character);
                }
                continue;
            }

            if character == '\'' || character == '"' {
                quote = Some(character);
                continue;
            }

            if character.is_whitespace() {
                break;
            }

            token.push(character);
        }

        token
    }
}
