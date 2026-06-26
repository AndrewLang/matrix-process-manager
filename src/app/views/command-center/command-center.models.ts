export type TerminalShell = "powerShell" | "cmd" | "zsh" | "bash";
export type TerminalOutputStream = "stdout" | "stderr" | "exit";

export interface TerminalStartRequest {
    shell: TerminalShell;
    workingDirectory?: string;
    cols?: number;
    rows?: number;
}

export interface TerminalStartResponse {
    sessionId: string;
    shell: TerminalShell;
    workingDirectory: string;
}

export interface TerminalOutputEvent {
    sessionId: string;
    stream: TerminalOutputStream;
    data: string;
}

export interface TerminalSegment {
    text: string;
    color: string;
    bold: boolean;
}

export interface TerminalTextPart {
    text: string;
    kind: "text" | "url" | "path";
}

export interface TerminalLine {
    id: number;
    segments: TerminalSegment[];
}

export interface TerminalUiTab {
    id: string;
    title: string;
}

export interface TerminalSessionExport {
    exportedAt: string;
    shell: TerminalShell;
    workingDirectory: string;
    lines: TerminalLine[];
}

export interface CommandAutocompleteSuggestion {
    commandId: number;
    commandLine: string;
    label: string;
    icon: string;
    description?: string;
    examples: CommandAutocompleteExample[];
    arguments: CommandAutocompleteArgument[];
    options: CommandAutocompleteOption[];
    usageCount: number;
    score: number;
    lastUsedAt?: string;
    frequentlyUsed: boolean;
    recentlyUsed: boolean;
}

export interface CommandAutocompleteExample {
    title: string;
    commandLine: string;
    description?: string;
}

export interface CommandAutocompleteArgument {
    name: string;
    description?: string;
    required: boolean;
}

export interface CommandAutocompleteOption {
    name: string;
    shortName?: string;
    description?: string;
    takesValue: boolean;
}

export interface CommandAutocompleteRequest {
    query: string;
    limit?: number;
}

export interface StartCommandExecutionRequest {
    commandLine: string;
    workingDirectory?: string;
    shell?: string;
}

export interface StartCommandExecutionResponse {
    historyId: number;
}

export interface FinishCommandExecutionRequest {
    historyId: number;
    exitCode?: number;
    durationMs?: number;
}
