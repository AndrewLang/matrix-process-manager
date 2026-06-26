import { Injectable, computed, signal } from "@angular/core";
import { UnlistenFn } from "@tauri-apps/api/event";
import { WorkareaStateService } from "../../../services/workarea-state.service";
import { CommandAutocompleteSuggestion, TerminalLine, TerminalOutputEvent, TerminalSegment, TerminalSessionExport, TerminalShell, TerminalUiTab } from "../command-center.models";
import { TerminalService } from "../services/terminal.service";

@Injectable()
export class CommandCenterViewModel {
    sessionId = signal<string | undefined>(undefined);
    shell = signal<TerminalShell>(this.defaultShell());
    workingDirectory = signal("");
    input = signal("");
    suggestions = signal<CommandAutocompleteSuggestion[]>([]);
    autocompleteOpen = signal(false);
    autocompleteLoading = signal(false);
    tabs = signal<TerminalUiTab[]>([{ id: "terminal-1", title: "Terminal 1" }]);
    activeTabId = signal("terminal-1");
    splitEnabled = signal(false);
    lines = signal<TerminalLine[]>([]);
    running = signal(false);
    status = computed(() => this.running() ? `Connected · ${this.shellLabel(this.shell())}` : "Disconnected");
    shellOptions = this.platformShells();

    private nextLineId = 1;
    private currentLineId?: number;
    private currentSegments: TerminalSegment[] = [];
    private unlisten?: UnlistenFn;
    private autocompleteRequest = 0;
    private pendingExecutions = new Map<string, { historyId: number; startedAt: number }>();
    private draggedTabId?: string;
    private autocompleteTimer?: ReturnType<typeof setTimeout>;
    private autocompleteCache = new Map<string, CommandAutocompleteSuggestion[]>();
    private suppressNextEcho = false;
    private echoBuffer = "";

    constructor(private terminal: TerminalService, private state: WorkareaStateService) { }

    async start(): Promise<void> {
        if (this.running()) {
            return;
        }

        this.unlisten = await this.terminal.onOutput((event) => this.receiveOutput(event));
        const response = await this.terminal.start({ shell: this.shell(), workingDirectory: this.workingDirectory() || undefined, cols: 120, rows: 32 });
        this.sessionId.set(response.sessionId);
        this.workingDirectory.set(response.workingDirectory);
        this.running.set(true);
        await this.terminal.resize(response.sessionId, 120, 32).catch(() => undefined);
    }

    async stop(): Promise<void> {
        const sessionId = this.sessionId();
        if (sessionId) {
            await this.terminal.stop(sessionId).catch(() => undefined);
        }

        this.running.set(false);
        this.sessionId.set(undefined);
        if (this.unlisten) {
            this.unlisten();
            this.unlisten = undefined;
        }
    }

    async submitInput(): Promise<void> {
        const sessionId = this.sessionId();
        const input = this.input();
        if (!sessionId || !input) {
            return;
        }

        this.input.set("");
        this.suggestions.set([]);
        this.autocompleteOpen.set(false);
        const execution = await this.terminal.startExecution({ commandLine: input, workingDirectory: this.workingDirectory(), shell: this.shell() });
        const token = this.nextExecutionToken();
        this.pendingExecutions.set(token, { historyId: execution.historyId, startedAt: Date.now() });
        this.suppressNextEcho = true;
        this.echoBuffer = "";
        await this.terminal.write(sessionId, this.commandWithCompletionMarker(input, token));
    }

    acceptSuggestion(suggestion: CommandAutocompleteSuggestion): void {
        this.input.set(`${suggestion.commandLine} `);
        this.suggestions.set([]);
        this.autocompleteOpen.set(false);
    }

    closeAutocomplete(): void {
        this.autocompleteOpen.set(false);
    }

    clearOutput(): void {
        this.lines.set([]);
        this.currentSegments = [];
        this.currentLineId = undefined;
    }

    async copyCommand(): Promise<void> {
        await navigator.clipboard.writeText(this.input()).catch(() => undefined);
    }

    async copyOutput(): Promise<void> {
        await navigator.clipboard.writeText(this.outputText()).catch(() => undefined);
    }

    exportSession(): void {
        const payload: TerminalSessionExport = {
            exportedAt: new Date().toISOString(),
            shell: this.shell(),
            workingDirectory: this.workingDirectory(),
            lines: this.lines(),
        };
        const url = URL.createObjectURL(new Blob([JSON.stringify(payload, null, 2)], { type: "application/json" }));
        const link = document.createElement("a");
        link.href = url;
        link.download = `matrix-terminal-${Date.now()}.json`;
        link.click();
        URL.revokeObjectURL(url);
    }

    async importHistory(file: File | undefined): Promise<void> {
        if (!file) {
            return;
        }
        const text = await file.text();
        try {
            const imported = JSON.parse(text) as Partial<TerminalSessionExport>;
            if (Array.isArray(imported.lines)) {
                this.lines.set(imported.lines.filter((line) => Array.isArray(line.segments)).slice(-5000));
                this.nextLineId = Math.max(0, ...this.lines().map((line) => line.id)) + 1;
                return;
            }
        } catch {
        }
        this.importPlainTextHistory(text);
    }

    toggleSplit(): void {
        this.splitEnabled.update((enabled) => !enabled);
    }

    duplicateTab(): void {
        const id = `terminal-${Date.now()}`;
        this.tabs.update((tabs) => [...tabs, { id, title: `Terminal ${tabs.length + 1}` }]);
        this.activeTabId.set(id);
    }

    closeTab(tabId: string): void {
        this.tabs.update((tabs) => tabs.length > 1 ? tabs.filter((tab) => tab.id !== tabId) : tabs);
        if (this.activeTabId() === tabId) {
            this.activeTabId.set(this.tabs()[0]?.id ?? "terminal-1");
        }
    }

    selectTab(tabId: string): void {
        this.activeTabId.set(tabId);
    }

    startTabDrag(tabId: string): void {
        this.draggedTabId = tabId;
    }

    dropTab(targetTabId: string): void {
        const draggedTabId = this.draggedTabId;
        this.draggedTabId = undefined;
        if (!draggedTabId || draggedTabId === targetTabId) {
            return;
        }
        this.tabs.update((tabs) => {
            const dragged = tabs.find((tab) => tab.id === draggedTabId);
            if (!dragged) {
                return tabs;
            }
            const withoutDragged = tabs.filter((tab) => tab.id !== draggedTabId);
            const targetIndex = withoutDragged.findIndex((tab) => tab.id === targetTabId);
            return [...withoutDragged.slice(0, targetIndex), dragged, ...withoutDragged.slice(targetIndex)];
        });
    }

    setShell(shell: TerminalShell): void {
        if (!this.running()) {
            this.shell.set(shell);
        }
    }

    setInput(input: string): void {
        this.input.set(input);
        this.queueAutocomplete(input);
    }

    private queueAutocomplete(input: string): void {
        if (this.autocompleteTimer) {
            clearTimeout(this.autocompleteTimer);
        }

        const query = input.trimStart();
        if (query.length === 0) {
            this.suggestions.set([]);
            this.autocompleteOpen.set(false);
            this.autocompleteLoading.set(false);
            return;
        }

        const cached = this.autocompleteCache.get(query);
        if (cached) {
            this.suggestions.set(cached);
            this.autocompleteOpen.set(cached.length > 0);
            this.autocompleteLoading.set(false);
            return;
        }

        const delay = Math.max(40, this.state.appSettings().terminalSettings.autocompleteDelayMs);
        this.autocompleteTimer = setTimeout(() => {
            this.searchAutocomplete(query).catch(() => undefined);
        }, delay);
    }

    private async searchAutocomplete(input: string): Promise<void> {
        const query = input.trimStart();
        const request = ++this.autocompleteRequest;
        if (query.length === 0) {
            this.suggestions.set([]);
            this.autocompleteOpen.set(false);
            this.autocompleteLoading.set(false);
            return;
        }

        this.autocompleteLoading.set(true);
        const suggestions = await this.terminal.autocomplete({ query, limit: 8 }).catch(() => []);
        if (request !== this.autocompleteRequest) {
            return;
        }

        this.suggestions.set(suggestions);
        this.autocompleteCache.set(query, suggestions);
        if (this.autocompleteCache.size > 80) {
            this.autocompleteCache.delete(this.autocompleteCache.keys().next().value ?? "");
        }
        this.autocompleteOpen.set(suggestions.length > 0);
        this.autocompleteLoading.set(false);
    }

    shellLabel(shell: TerminalShell): string {
        switch (shell) {
            case "powerShell":
                return "PowerShell";
            case "cmd":
                return "CMD";
            case "zsh":
                return "zsh";
            case "bash":
                return "bash";
        }
    }

    private receiveOutput(event: TerminalOutputEvent): void {
        if (event.sessionId !== this.sessionId() && this.sessionId()) {
            return;
        }

        if (event.stream === "exit") {
            this.running.set(false);
            return;
        }

        const data = this.consumeCompletionMarkers(this.consumeEcho(event.data));
        if (!data) {
            return;
        }

        this.appendAnsi(data, event.stream === "stderr" ? "text-(--red)" : "text-[#dce7f0]");
    }

    private consumeEcho(data: string): string {
        if (!this.suppressNextEcho) {
            return data;
        }

        this.echoBuffer += data;
        const cleaned = this.stripAnsi(this.echoBuffer).replace(/\r/g, "");
        const newlineIndex = cleaned.indexOf("\n");
        if (newlineIndex < 0) {
            return "";
        }

        this.suppressNextEcho = false;
        this.echoBuffer = "";
        return cleaned.slice(newlineIndex + 1);
    }

    private consumeCompletionMarkers(data: string): string {
        return data.replace(/\r?\n?__MPM_EXIT:([^:]+):(-?\d+)\r?\n?/g, (_match, token: string, exitCode: string) => {
            const execution = this.pendingExecutions.get(token);
            if (execution) {
                this.pendingExecutions.delete(token);
                this.terminal.finishExecution({ historyId: execution.historyId, exitCode: Number(exitCode), durationMs: Date.now() - execution.startedAt }).catch(() => undefined);
            }
            return "";
        });
    }

    private commandWithCompletionMarker(input: string, token: string): string {
        switch (this.shell()) {
            case "powerShell":
                return `${input}; $mpmOk = $?; $mpmNative = $global:LASTEXITCODE; $mpmExit = if ($null -ne $mpmNative) { $mpmNative } elseif ($mpmOk) { 0 } else { 1 }; Write-Output "__MPM_EXIT:${token}:$mpmExit"\r`;
            case "cmd":
                return `${input} & echo __MPM_EXIT:${token}:%ERRORLEVEL%\r`;
            case "zsh":
            case "bash":
                return `${input}; printf '\\n__MPM_EXIT:${token}:%s\\n' "$?"\r`;
        }
    }

    private nextExecutionToken(): string {
        return `${Date.now()}${Math.random().toString(36).slice(2)}`;
    }

    private outputText(): string {
        return this.lines().map((line) => line.segments.map((segment) => segment.text).join("")).join("\n");
    }

    private importPlainTextHistory(text: string): void {
        const lines = text.split(/\r?\n/).map((line) => ({ id: this.nextLineId++, segments: [{ text: line, color: "text-[#dce7f0]", bold: false }] }));
        this.lines.set(lines.slice(-5000));
    }

    private appendAnsi(data: string, fallbackColor: string): void {
        let color = fallbackColor;
        let bold = false;
        const chunks = data.split(/(\u001b\[[?0-9;]*[ -/]*[@-~]|\r?\n)/g).filter((chunk) => chunk.length > 0 && chunk !== "\r");

        for (const chunk of chunks) {
            if (chunk === "\n" || chunk === "\r\n") {
                this.commitLine();
                continue;
            }

            if (chunk.startsWith("\u001b[")) {
                if (!chunk.endsWith("m")) {
                    continue;
                }

                const codes = chunk.slice(2, -1).split(";").map((code) => Number(code || 0));
                for (const code of codes) {
                    if (code === 0) {
                        color = fallbackColor;
                        bold = false;
                    } else if (code === 1) {
                        bold = true;
                    } else if (code >= 30 && code <= 37 || code >= 90 && code <= 97) {
                        color = this.ansiColor(code);
                    }
                }
                continue;
            }

            this.currentSegments.push({ text: chunk, color, bold });
        }

        this.flushCurrentLine();
    }

    private stripAnsi(data: string): string {
        return data.replace(/\u001b\[[?0-9;]*[ -/]*[@-~]/g, "");
    }

    private commitLine(): void {
        const segments = this.currentSegments.length > 0 ? this.currentSegments : [{ text: "", color: "text-[#dce7f0]", bold: false }];
        const id = this.currentLineId ?? this.nextLineId++;
        this.lines.update((lines) => {
            const next = lines.at(-1)?.id === id ? [...lines.slice(0, -1), { id, segments }] : [...lines, { id, segments }];
            return next.slice(-600);
        });
        this.currentSegments = [];
        this.currentLineId = undefined;
    }

    private flushCurrentLine(): void {
        if (this.currentSegments.length === 0) {
            return;
        }

        const id = this.currentLineId ?? this.nextLineId++;
        this.currentLineId = id;
        this.lines.update((lines) => lines.at(-1)?.id === id ? [...lines.slice(0, -1), { id, segments: [...this.currentSegments] }] : [...lines, { id, segments: [...this.currentSegments] }]);
    }

    private ansiColor(code: number): string {
        const colors: Record<number, string> = {
            30: "text-[#8796a5]",
            31: "text-(--red)",
            32: "text-(--green)",
            33: "text-(--orange)",
            34: "text-[#65b8ff]",
            35: "text-[#d68cff]",
            36: "text-[#4dd7ef]",
            37: "text-[#dce7f0]",
            90: "text-[#7f91a4]",
            91: "text-[#ff827a]",
            92: "text-[#67df9b]",
            93: "text-[#ffd166]",
            94: "text-[#8cc8ff]",
            95: "text-[#e0a7ff]",
            96: "text-[#82efff]",
            97: "text-white",
        };
        return colors[code] ?? "text-[#dce7f0]";
    }

    private defaultShell(): TerminalShell {
        return navigator.platform.toLowerCase().includes("win") ? "powerShell" : "zsh";
    }

    private platformShells(): TerminalShell[] {
        return navigator.platform.toLowerCase().includes("win") ? ["powerShell", "cmd"] : ["zsh", "bash"];
    }
}
