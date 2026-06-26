import { Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { CommandAutocompleteRequest, CommandAutocompleteSuggestion, FinishCommandExecutionRequest, StartCommandExecutionRequest, StartCommandExecutionResponse, TerminalOutputEvent, TerminalStartRequest, TerminalStartResponse } from "../command-center.models";

@Injectable()
export class TerminalService {
    start(request: TerminalStartRequest): Promise<TerminalStartResponse> {
        return invoke<TerminalStartResponse>("start_terminal_session", { request });
    }

    write(sessionId: string, input: string): Promise<void> {
        return invoke<void>("write_terminal_input", { request: { sessionId, input } });
    }

    resize(sessionId: string, cols: number, rows: number): Promise<void> {
        return invoke<void>("resize_terminal_session", { request: { sessionId, cols, rows } });
    }

    stop(sessionId: string): Promise<void> {
        return invoke<void>("stop_terminal_session", { request: { sessionId } });
    }

    autocomplete(request: CommandAutocompleteRequest): Promise<CommandAutocompleteSuggestion[]> {
        return invoke<CommandAutocompleteSuggestion[]>("autocomplete_commands", { request });
    }

    startExecution(request: StartCommandExecutionRequest): Promise<StartCommandExecutionResponse> {
        return invoke<StartCommandExecutionResponse>("start_command_execution", { request });
    }

    finishExecution(request: FinishCommandExecutionRequest): Promise<void> {
        return invoke<void>("finish_command_execution", { request });
    }

    onOutput(handler: (event: TerminalOutputEvent) => void): Promise<UnlistenFn> {
        return listen<TerminalOutputEvent>("terminal://output", (event) => handler(event.payload));
    }
}
