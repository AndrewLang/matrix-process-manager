import { Component, HostListener, OnInit, computed, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { PortScan, PortUsage } from "../../app.models";

type ProtocolFilter = "all" | "tcp" | "udp";
type StateFilter = "all" | "listening" | "established";
type PortSortDirection = "asc" | "desc";
type PortColumnKey = "localPort" | "protocol" | "localAddress" | "remote" | "state" | "processName" | "pid";

interface PortColumn {
    key: PortColumnKey;
    label: string;
    width: number;
    minWidth: number;
    align?: "left" | "right";
}

@Component({
    selector: "mtx-ports-view",
    templateUrl: "./ports-view.component.html",
})
export class PortsViewComponent implements OnInit {
    scan = signal<PortScan | undefined>(undefined);
    selectedKey = signal("");
    query = signal("");
    protocolFilter = signal<ProtocolFilter>("all");
    stateFilter = signal<StateFilter>("all");
    loading = signal(false);
    error = signal("");
    actionMessage = signal("");
    sortKey = signal<PortColumnKey>("localPort");
    sortDirection = signal<PortSortDirection>("asc");
    columns = signal<PortColumn[]>([
        { key: "localPort", label: "Port", width: 92, minWidth: 80 },
        { key: "protocol", label: "Protocol", width: 96, minWidth: 82 },
        { key: "localAddress", label: "Local Address", width: 170, minWidth: 128 },
        { key: "remote", label: "Remote", width: 190, minWidth: 132 },
        { key: "state", label: "State", width: 122, minWidth: 92 },
        { key: "processName", label: "App", width: 190, minWidth: 130 },
        { key: "pid", label: "PID", width: 88, minWidth: 72, align: "right" },
    ]);

    ports = computed(() => this.scan()?.ports ?? []);
    filteredPorts = computed(() => this.filterPorts());
    tableWidth = computed(() => this.columns().reduce((total, column) => total + column.width, 0));
    selectedPort = computed(() => this.filteredPorts().find((port) => this.portKey(port) === this.selectedKey()) ?? this.filteredPorts()[0]);
    listeningCount = computed(() => this.ports().filter((port) => this.isListening(port)).length);
    establishedCount = computed(() => this.ports().filter((port) => port.state.toLowerCase() === "established").length);
    appCount = computed(() => new Set(this.ports().map((port) => `${port.pid ?? "none"}:${port.processName}`)).size);
    publicCount = computed(() => this.ports().filter((port) => port.localAddress === "All interfaces").length);

    private resizing?: { index: number; startX: number; startWidth: number };

    ngOnInit(): void {
        this.refresh();
    }

    refresh(): void {
        this.loading.set(true);
        this.error.set("");
        this.actionMessage.set("");
        invoke<PortScan>("get_port_scan")
            .then((scan) => {
                this.scan.set(scan);
                const selected = scan.ports.find((port) => this.portKey(port) === this.selectedKey()) ?? scan.ports[0];
                this.selectedKey.set(selected ? this.portKey(selected) : "");
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Port scan failed."))
            .finally(() => this.loading.set(false));
    }

    selectPort(port: PortUsage): void {
        this.selectedKey.set(this.portKey(port));
        this.actionMessage.set("");
    }

    setQuery(value: string): void {
        this.query.set(value);
    }

    setProtocolFilter(filter: ProtocolFilter): void {
        this.protocolFilter.set(filter);
    }

    setStateFilter(filter: StateFilter): void {
        this.stateFilter.set(filter);
    }

    sortBy(column: PortColumn): void {
        if (this.resizing) {
            return;
        }

        if (this.sortKey() === column.key) {
            this.sortDirection.update((direction) => direction === "asc" ? "desc" : "asc");
            return;
        }

        this.sortKey.set(column.key);
        this.sortDirection.set("asc");
    }

    sortIcon(column: PortColumn): string {
        if (this.sortKey() !== column.key) {
            return "bi-chevron-expand";
        }

        return this.sortDirection() === "asc" ? "bi-chevron-up" : "bi-chevron-down";
    }

    startResize(event: MouseEvent, index: number): void {
        event.preventDefault();
        event.stopPropagation();
        const column = this.columns()[index];
        this.resizing = { index, startX: event.clientX, startWidth: column.width };
    }

    copyEndpoint(port: PortUsage | undefined): void {
        if (!port) {
            return;
        }

        navigator.clipboard.writeText(this.endpoint(port)).then(
            () => this.actionMessage.set("Endpoint copied."),
            () => this.actionMessage.set("Endpoint could not be copied."),
        );
    }

    openLocalhost(port: PortUsage | undefined): void {
        if (!port || port.protocol !== "TCP") {
            return;
        }

        openUrl(`http://localhost:${port.localPort}`).catch(() => this.actionMessage.set("Localhost URL could not be opened."));
    }

    openProcessLocation(port: PortUsage | undefined): void {
        if (!port?.processPath) {
            return;
        }

        revealItemInDir(port.processPath).catch(() => this.actionMessage.set("Process location could not be opened."));
    }

    endProcess(port: PortUsage | undefined): void {
        if (!port?.pid || this.loading()) {
            return;
        }

        this.loading.set(true);
        this.actionMessage.set("");
        invoke<void>("terminate_process", { pid: port.pid })
            .then(() => {
                this.actionMessage.set(`${port.processName} ended.`);
                this.refresh();
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Process could not be ended."))
            .finally(() => this.loading.set(false));
    }

    endpoint(port: PortUsage): string {
        return `${port.localAddress}:${port.localPort}`;
    }

    remoteEndpoint(port: PortUsage): string {
        if (!port.remoteAddress || !port.remotePort || port.remoteAddress === "0.0.0.0" || port.remoteAddress === "::") {
            return "-";
        }

        return `${port.remoteAddress}:${port.remotePort}`;
    }

    portKey(port: PortUsage): string {
        return `${port.protocol}:${port.localAddress}:${port.localPort}:${port.remoteAddress ?? ""}:${port.remotePort ?? ""}:${port.pid ?? ""}`;
    }

    isSelected(port: PortUsage): boolean {
        return this.portKey(port) === this.selectedKey();
    }

    scannedAtText(): string {
        const scannedAt = Number(this.scan()?.scannedAt ?? 0);
        return scannedAt > 0 ? new Date(scannedAt * 1000).toLocaleTimeString() : "Not scanned";
    }

    private filterPorts(): PortUsage[] {
        const query = this.query().trim().toLowerCase();
        const protocol = this.protocolFilter();
        const state = this.stateFilter();

        return [...this.ports()]
            .filter((port) => protocol === "all" || port.protocol.toLowerCase() === protocol)
            .filter((port) => state === "all" || state === "listening" && this.isListening(port) || state === "established" && port.state.toLowerCase() === "established")
            .filter((port) => {
                if (!query) {
                    return true;
                }

                return [
                    port.protocol,
                    port.localAddress,
                    port.localPort.toString(),
                    port.remoteAddress ?? "",
                    port.remotePort?.toString() ?? "",
                    port.state,
                    port.pid?.toString() ?? "",
                    port.processName,
                    port.processPath ?? "",
                ].some((value) => value.toLowerCase().includes(query));
            })
            .sort((left, right) => this.comparePorts(left, right));
    }

    private comparePorts(left: PortUsage, right: PortUsage): number {
        const direction = this.sortDirection() === "asc" ? 1 : -1;
        let result = 0;

        switch (this.sortKey()) {
            case "localPort":
                result = left.localPort - right.localPort;
                break;
            case "pid":
                result = (left.pid ?? -1) - (right.pid ?? -1);
                break;
            case "remote":
                result = this.remoteEndpoint(left).localeCompare(this.remoteEndpoint(right));
                break;
            case "protocol":
                result = left.protocol.localeCompare(right.protocol);
                break;
            case "localAddress":
                result = left.localAddress.localeCompare(right.localAddress);
                break;
            case "state":
                result = left.state.localeCompare(right.state);
                break;
            case "processName":
                result = left.processName.localeCompare(right.processName);
                break;
        }

        return (result || left.localPort - right.localPort || left.protocol.localeCompare(right.protocol)) * direction;
    }

    private isListening(port: PortUsage): boolean {
        const state = port.state.toLowerCase();
        return state === "listen" || state === "listening" || state === "open";
    }

    @HostListener("document:mousemove", ["$event"])
    resizeColumn(event: MouseEvent): void {
        if (!this.resizing) {
            return;
        }

        const { index, startX, startWidth } = this.resizing;
        this.columns.update((columns) =>
            columns.map((column, columnIndex) =>
                columnIndex === index
                    ? { ...column, width: Math.max(column.minWidth, startWidth + event.clientX - startX) }
                    : column,
            ),
        );
    }

    @HostListener("document:mouseup")
    stopResize(): void {
        this.resizing = undefined;
    }
}
