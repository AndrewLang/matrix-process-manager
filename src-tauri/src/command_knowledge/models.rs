use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationRecord {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub last_scanned_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewApplicationRecord {
    pub name: String,
    pub path: String,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRecord {
    pub id: i64,
    pub application_id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCommandRecord {
    pub application_id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOptionRecord {
    pub id: i64,
    pub command_id: i64,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub takes_value: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCommandOptionRecord {
    pub command_id: i64,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub takes_value: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandArgumentRecord {
    pub id: i64,
    pub command_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub position: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCommandArgumentRecord {
    pub command_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub position: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleRecord {
    pub id: i64,
    pub command_id: i64,
    pub title: String,
    pub command_line: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExampleRecord {
    pub command_id: i64,
    pub title: String,
    pub command_line: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    pub id: i64,
    pub command_id: Option<i64>,
    pub command_line: String,
    pub working_directory: Option<String>,
    pub shell: Option<String>,
    pub exit_code: Option<i64>,
    pub duration_ms: Option<i64>,
    pub executed_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewHistoryRecord {
    pub command_id: Option<i64>,
    pub command_line: String,
    pub working_directory: Option<String>,
    pub shell: Option<String>,
    pub exit_code: Option<i64>,
    pub duration_ms: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageStatisticRecord {
    pub id: i64,
    pub command_id: i64,
    pub run_count: i64,
    pub last_used_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUsageStatisticRecord {
    pub command_id: i64,
    pub run_count: i64,
    pub last_used_at: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApplicationScanResult {
    pub scanned: usize,
    pub inserted: usize,
    pub updated: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandIndexResult {
    pub applications: usize,
    pub commands: usize,
    pub examples: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandAutocompleteRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandAutocompleteSuggestion {
    pub command_id: i64,
    pub command_line: String,
    pub label: String,
    pub icon: String,
    pub description: Option<String>,
    pub examples: Vec<CommandAutocompleteExample>,
    pub arguments: Vec<CommandAutocompleteArgument>,
    pub options: Vec<CommandAutocompleteOption>,
    pub usage_count: i64,
    pub score: i64,
    pub last_used_at: Option<String>,
    pub frequently_used: bool,
    pub recently_used: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandAutocompleteExample {
    pub title: String,
    pub command_line: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandAutocompleteArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandAutocompleteOption {
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub takes_value: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartCommandExecutionRequest {
    pub command_line: String,
    pub working_directory: Option<String>,
    pub shell: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartCommandExecutionResponse {
    pub history_id: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishCommandExecutionRequest {
    pub history_id: i64,
    pub exit_code: Option<i64>,
    pub duration_ms: Option<i64>,
}
