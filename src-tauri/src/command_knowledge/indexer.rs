use crate::command_knowledge::models::{
    ApplicationRecord, CommandIndexResult, NewCommandOptionRecord, NewCommandRecord,
    NewExampleRecord,
};
use crate::command_knowledge::repositories::{
    ApplicationRepository, CommandOptionRepository, CommandRepository, ExampleRepository,
};
use crate::models::CommandError;
use std::collections::{HashSet, VecDeque};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub trait CommandParser: Send + Sync {
    fn score(&self, application: &ApplicationRecord) -> u8;
    fn parse(&self, command_path: &[String], output: &str) -> ParsedCommandHelp;
}

#[derive(Clone, Debug, Default)]
pub struct ParsedCommandHelp {
    pub description: Option<String>,
    pub subcommands: Vec<ParsedSubcommand>,
    pub options: Vec<ParsedOption>,
    pub examples: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ParsedSubcommand {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ParsedOption {
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub takes_value: bool,
}

pub struct CommandIndexer<'a> {
    applications: &'a ApplicationRepository,
    commands: &'a CommandRepository,
    command_options: &'a CommandOptionRepository,
    examples: &'a ExampleRepository,
    parsers: Vec<Box<dyn CommandParser>>,
    max_depth: usize,
    command_timeout: Duration,
}

impl<'a> CommandIndexer<'a> {
    pub fn new(
        applications: &'a ApplicationRepository,
        commands: &'a CommandRepository,
        command_options: &'a CommandOptionRepository,
        examples: &'a ExampleRepository,
    ) -> Self {
        Self {
            applications,
            commands,
            command_options,
            examples,
            parsers: vec![
                Box::new(DockerParser::new()),
                Box::new(GitParser::new()),
                Box::new(CargoParser::new()),
                Box::new(PnpmParser::new()),
                Box::new(GenericParser::new()),
            ],
            max_depth: 4,
            command_timeout: Duration::from_millis(1200),
        }
    }

    pub fn index(&self) -> Result<CommandIndexResult, CommandError> {
        let applications = self.applications.list()?;
        let mut commands = 0;
        let mut examples = 0;

        for application in &applications {
            let result = self.index_application(application)?;
            commands += result.commands;
            examples += result.examples;
        }

        Ok(CommandIndexResult {
            applications: applications.len(),
            commands,
            examples,
        })
    }

    pub fn index_application_overview(
        &self,
        application: &ApplicationRecord,
    ) -> Result<CommandIndexResult, CommandError> {
        let Some(output) = self.read_help(application, &[]) else {
            return Ok(CommandIndexResult {
                applications: 1,
                commands: 0,
                examples: 0,
            });
        };

        let parsed = self.parser_for(application).parse(&[], &output);
        let command = self.commands.upsert(&NewCommandRecord {
            application_id: application.id,
            name: application.name.clone(),
            description: parsed.description.clone(),
        })?;
        let mut commands = 1;
        let mut examples = 0;

        for option in parsed.options {
            self.command_options.upsert(&NewCommandOptionRecord {
                command_id: command.id,
                name: option.name,
                short_name: option.short_name,
                description: option.description,
                takes_value: option.takes_value,
            })?;
        }

        for (index, example) in parsed.examples.into_iter().enumerate() {
            self.examples.upsert(&NewExampleRecord {
                command_id: command.id,
                title: format!("Example {}", index + 1),
                command_line: example,
                description: None,
            })?;
            examples += 1;
        }

        for subcommand in parsed.subcommands {
            self.commands.upsert(&NewCommandRecord {
                application_id: application.id,
                name: self.command_name(application, &[subcommand.name]),
                description: subcommand.description,
            })?;
            commands += 1;
        }

        Ok(CommandIndexResult {
            applications: 1,
            commands,
            examples,
        })
    }

    fn index_application(
        &self,
        application: &ApplicationRecord,
    ) -> Result<CommandIndexResult, CommandError> {
        let mut queue = VecDeque::from([Vec::<String>::new()]);
        let mut visited = HashSet::new();
        let mut commands = 0;
        let mut examples = 0;

        while let Some(command_path) = queue.pop_front() {
            if command_path.len() > self.max_depth || !visited.insert(command_path.join(" ")) {
                continue;
            }

            let Some(output) = self.read_help(application, &command_path) else {
                continue;
            };

            let parsed = self.parser_for(application).parse(&command_path, &output);
            let command_name = self.command_name(application, &command_path);
            let command = self.commands.upsert(&NewCommandRecord {
                application_id: application.id,
                name: command_name.clone(),
                description: parsed.description.clone(),
            })?;
            commands += 1;

            for option in parsed.options {
                self.command_options.upsert(&NewCommandOptionRecord {
                    command_id: command.id,
                    name: option.name,
                    short_name: option.short_name,
                    description: option.description,
                    takes_value: option.takes_value,
                })?;
            }

            for (index, example) in parsed.examples.into_iter().enumerate() {
                self.examples.upsert(&NewExampleRecord {
                    command_id: command.id,
                    title: format!("Example {}", index + 1),
                    command_line: example,
                    description: None,
                })?;
                examples += 1;
            }

            if command_path.len() < self.max_depth {
                for subcommand in parsed.subcommands {
                    let mut child = command_path.clone();
                    child.push(subcommand.name.clone());
                    self.commands.upsert(&NewCommandRecord {
                        application_id: application.id,
                        name: self.command_name(application, &child),
                        description: subcommand.description,
                    })?;
                    queue.push_back(child);
                }
            }
        }

        Ok(CommandIndexResult {
            applications: 1,
            commands,
            examples,
        })
    }

    fn parser_for(&self, application: &ApplicationRecord) -> &dyn CommandParser {
        self.parsers
            .iter()
            .max_by_key(|parser| parser.score(application))
            .map(|parser| parser.as_ref())
            .unwrap_or_else(|| self.parsers[0].as_ref())
    }

    fn command_name(&self, application: &ApplicationRecord, command_path: &[String]) -> String {
        if command_path.is_empty() {
            application.name.clone()
        } else {
            format!("{} {}", application.name, command_path.join(" "))
        }
    }

    fn read_help(
        &self,
        application: &ApplicationRecord,
        command_path: &[String],
    ) -> Option<String> {
        for arguments in self.help_attempts(command_path) {
            if let Some(output) = self.run_help_command(&application.path, &arguments) {
                if self.looks_like_help(&output) {
                    return Some(output);
                }
            }
        }
        None
    }

    fn help_attempts(&self, command_path: &[String]) -> Vec<Vec<String>> {
        let mut attempts = Vec::new();
        let mut long_help = command_path.to_vec();
        long_help.push("--help".to_string());
        attempts.push(long_help);

        let mut help = command_path.to_vec();
        help.push("help".to_string());
        attempts.push(help);

        let mut short_help = command_path.to_vec();
        short_help.push("-h".to_string());
        attempts.push(short_help);

        attempts
    }

    fn run_help_command(&self, executable: &str, arguments: &[String]) -> Option<String> {
        let mut child = Command::new(executable)
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .ok()?;

        let start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if start.elapsed() >= self.command_timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                Ok(None) => thread::sleep(Duration::from_millis(20)),
                Err(_) => return None,
            }
        }

        child.wait_with_output().ok().and_then(|output| {
            let mut data = String::new();
            data.push_str(&String::from_utf8_lossy(&output.stdout));
            if data.trim().is_empty() {
                data.push_str(&String::from_utf8_lossy(&output.stderr));
            }
            let data = data.trim().to_string();
            (!data.is_empty()).then_some(data)
        })
    }

    fn looks_like_help(&self, output: &str) -> bool {
        let output = output.to_ascii_lowercase();
        [
            "usage:",
            "options:",
            "commands:",
            "subcommands:",
            "examples:",
            "available commands",
        ]
        .iter()
        .any(|token| output.contains(token))
    }
}

pub struct GenericParser;

impl GenericParser {
    pub fn new() -> Self {
        Self
    }

    fn application_name(application: &ApplicationRecord) -> String {
        std::path::Path::new(&application.path)
            .file_stem()
            .or_else(|| std::path::Path::new(&application.path).file_name())
            .map(|name| name.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_else(|| application.name.to_ascii_lowercase())
    }

    fn application_matches(application: &ApplicationRecord, expected: &str) -> bool {
        application.name.eq_ignore_ascii_case(expected)
            || Self::application_name(application) == expected
    }

    fn section_name(&self, line: &str) -> Option<HelpSection> {
        let normalized = line.trim().trim_end_matches(':').to_ascii_lowercase();
        match normalized.as_str() {
            "commands"
            | "subcommands"
            | "available commands"
            | "available subcommands"
            | "management commands"
            | "common commands" => Some(HelpSection::Commands),
            "options" | "flags" | "global options" | "global flags" => Some(HelpSection::Options),
            "examples" | "example" => Some(HelpSection::Examples),
            _ if normalized.contains("common") && normalized.contains("commands") => {
                Some(HelpSection::Commands)
            }
            _ if normalized.contains("available") && normalized.contains("commands") => {
                Some(HelpSection::Commands)
            }
            _ => None,
        }
    }

    fn parse_description(&self, output: &str) -> Option<String> {
        output
            .lines()
            .map(str::trim)
            .find(|line| {
                !line.is_empty()
                    && !line.to_ascii_lowercase().starts_with("usage:")
                    && self.section_name(line).is_none()
            })
            .map(|line| line.chars().take(512).collect())
    }

    fn parse_subcommand(&self, line: &str) -> Option<ParsedSubcommand> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('-') {
            return None;
        }

        let (name, description) = self.split_name_description(line)?;
        if !self.is_command_name(&name) {
            return None;
        }

        Some(ParsedSubcommand { name, description })
    }

    fn parse_option(&self, line: &str) -> Option<ParsedOption> {
        let line = line.trim();
        if !line.starts_with('-') {
            return None;
        }

        let (names, description) = self.split_name_description(line)?;
        let mut long_name = None;
        let mut short_name = None;
        let mut takes_value = false;

        for raw in names.split(',') {
            let token = raw.trim();
            let name = token
                .split(|character: char| character.is_whitespace() || character == '=')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            if name.is_empty() {
                continue;
            }
            takes_value |= token.contains('<')
                || token.contains('=')
                || token.contains(" VALUE")
                || token.contains(" value")
                || token.contains(" FILE")
                || token.contains(" file");
            if name.starts_with("--") {
                long_name = Some(name);
            } else if name.starts_with('-') {
                short_name = Some(name);
            }
        }

        long_name
            .or_else(|| short_name.clone())
            .map(|name| ParsedOption {
                name,
                short_name,
                description,
                takes_value,
            })
    }

    fn split_name_description(&self, line: &str) -> Option<(String, Option<String>)> {
        let mut whitespace_run = 0;
        let mut split_at = None;
        for (index, character) in line.char_indices() {
            if character.is_whitespace() {
                whitespace_run += 1;
                if whitespace_run >= 2 {
                    split_at = Some(index + 1 - whitespace_run);
                    break;
                }
            } else {
                whitespace_run = 0;
            }
        }

        match split_at {
            Some(index) => {
                let name = line[..index].trim().to_string();
                let description = line[index..].trim().to_string();
                (!name.is_empty()).then_some((
                    name,
                    (!description.is_empty()).then_some(description.chars().take(512).collect()),
                ))
            }
            None => {
                let mut parts = line.splitn(2, char::is_whitespace);
                let name = parts.next()?.trim().to_string();
                let description = parts
                    .next()
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                (!name.is_empty()).then_some((
                    name,
                    description.map(|value| value.chars().take(512).collect()),
                ))
            }
        }
    }

    fn is_command_name(&self, name: &str) -> bool {
        !name.contains(',')
            && !name.contains('[')
            && !name.contains('<')
            && !name.contains('>')
            && name.chars().all(|character| {
                character.is_alphanumeric() || matches!(character, '-' | '_' | ':')
            })
    }
}

impl CommandParser for GenericParser {
    fn score(&self, _: &ApplicationRecord) -> u8 {
        1
    }

    fn parse(&self, _: &[String], output: &str) -> ParsedCommandHelp {
        let mut parsed = ParsedCommandHelp {
            description: self.parse_description(output),
            ..ParsedCommandHelp::default()
        };
        let mut section = HelpSection::Unknown;

        for line in output.lines() {
            if let Some(next_section) = self.section_name(line) {
                section = next_section;
                continue;
            }

            match section {
                HelpSection::Commands => {
                    if let Some(subcommand) = self.parse_subcommand(line) {
                        parsed.subcommands.push(subcommand);
                    }
                }
                HelpSection::Options => {
                    if let Some(option) = self.parse_option(line) {
                        parsed.options.push(option);
                    }
                }
                HelpSection::Examples => {
                    let example = line.trim();
                    if !example.is_empty() && !example.ends_with(':') {
                        parsed.examples.push(example.chars().take(1024).collect());
                    }
                }
                HelpSection::Unknown => {
                    if let Some(option) = self.parse_option(line) {
                        parsed.options.push(option);
                    }
                }
            }
        }

        parsed
    }
}

pub struct DockerParser {
    generic: GenericParser,
}

impl DockerParser {
    pub fn new() -> Self {
        Self {
            generic: GenericParser::new(),
        }
    }
}

impl CommandParser for DockerParser {
    fn score(&self, application: &ApplicationRecord) -> u8 {
        if GenericParser::application_matches(application, "docker") {
            100
        } else {
            0
        }
    }

    fn parse(&self, command_path: &[String], output: &str) -> ParsedCommandHelp {
        self.generic.parse(command_path, output)
    }
}

pub struct GitParser {
    generic: GenericParser,
}

impl GitParser {
    pub fn new() -> Self {
        Self {
            generic: GenericParser::new(),
        }
    }
}

impl CommandParser for GitParser {
    fn score(&self, application: &ApplicationRecord) -> u8 {
        if GenericParser::application_matches(application, "git") {
            100
        } else {
            0
        }
    }

    fn parse(&self, command_path: &[String], output: &str) -> ParsedCommandHelp {
        self.generic.parse(command_path, output)
    }
}

pub struct CargoParser {
    generic: GenericParser,
}

impl CargoParser {
    pub fn new() -> Self {
        Self {
            generic: GenericParser::new(),
        }
    }
}

impl CommandParser for CargoParser {
    fn score(&self, application: &ApplicationRecord) -> u8 {
        if GenericParser::application_matches(application, "cargo") {
            100
        } else {
            0
        }
    }

    fn parse(&self, command_path: &[String], output: &str) -> ParsedCommandHelp {
        self.generic.parse(command_path, output)
    }
}

pub struct PnpmParser {
    generic: GenericParser,
}

impl PnpmParser {
    pub fn new() -> Self {
        Self {
            generic: GenericParser::new(),
        }
    }

    fn is_command_section(&self, line: &str) -> bool {
        matches!(
            line.trim()
                .trim_end_matches(':')
                .to_ascii_lowercase()
                .as_str(),
            "manage your dependencies" | "review your dependencies" | "run your scripts" | "other"
        )
    }

    fn is_stop_section(&self, line: &str) -> bool {
        matches!(
            line.trim()
                .trim_end_matches(':')
                .to_ascii_lowercase()
                .as_str(),
            "options" | "usage" | "flags" | "global options"
        )
    }

    fn parse_subcommands(&self, line: &str) -> Vec<ParsedSubcommand> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('-') {
            return Vec::new();
        }

        let Some((names, description)) = self.split_command_description(line) else {
            return Vec::new();
        };

        names
            .split(',')
            .filter_map(|name| {
                let name = name.trim().to_string();
                self.generic
                    .is_command_name(&name)
                    .then_some(ParsedSubcommand {
                        name,
                        description: description.clone(),
                    })
            })
            .collect()
    }

    fn split_command_description(&self, line: &str) -> Option<(String, Option<String>)> {
        let mut whitespace_run = 0;
        let mut split_at = None;
        for (index, character) in line.char_indices() {
            if character.is_whitespace() {
                whitespace_run += 1;
                if whitespace_run >= 2 {
                    split_at = Some(index + 1 - whitespace_run);
                    break;
                }
            } else {
                whitespace_run = 0;
            }
        }

        let index = split_at?;
        let names = line[..index].trim().to_string();
        let description = line[index..].trim().to_string();
        (!names.is_empty()).then_some((
            names,
            (!description.is_empty()).then_some(description.chars().take(512).collect()),
        ))
    }
}

impl CommandParser for PnpmParser {
    fn score(&self, application: &ApplicationRecord) -> u8 {
        if GenericParser::application_matches(application, "pnpm") {
            100
        } else {
            0
        }
    }

    fn parse(&self, command_path: &[String], output: &str) -> ParsedCommandHelp {
        let mut parsed = self.generic.parse(command_path, output);
        let mut command_section = false;
        let mut names = HashSet::new();
        parsed
            .subcommands
            .retain(|subcommand| names.insert(subcommand.name.clone()));

        for line in output.lines() {
            if self.is_command_section(line) {
                command_section = true;
                continue;
            }

            if self.is_stop_section(line) {
                command_section = false;
                continue;
            }

            if command_section {
                for subcommand in self.parse_subcommands(line) {
                    if names.insert(subcommand.name.clone()) {
                        parsed.subcommands.push(subcommand);
                    }
                }
            }
        }

        parsed
    }
}

#[derive(Clone, Copy, Debug)]
enum HelpSection {
    Unknown,
    Commands,
    Options,
    Examples,
}
