import { Component, OnInit, computed, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { PortScan, PortUsage } from "../../app.models";
import { DataGridColumn, DataGridComponent } from "../../components/data-grid/data-grid.component";
import { SelectComponent } from "../../components/select/select.component";

type ProtocolFilter = "all" | "tcp" | "udp";
type StateFilter = "all" | "listening" | "established";

@Component({
    selector: "mtx-ports-view",
    imports: [DataGridComponent, SelectComponent],
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
    columns: DataGridColumn<PortUsage>[] = [
        { key: "localPort", label: "Port", width: 92, minWidth: 80, value: (port) => port.localPort, cellClass: () => "font-mono text-[#e6f0fa]" },
        { key: "protocol", label: "Protocol", width: 96, minWidth: 82, value: (port) => port.protocol },
        { key: "localAddress", label: "Local Address", width: 170, minWidth: 128, value: (port) => port.localAddress, cellClass: () => "font-mono" },
        { key: "remote", label: "Remote", width: 190, minWidth: 132, value: (port) => this.remoteEndpoint(port), cellClass: () => "font-mono text-(--muted)" },
        { key: "state", label: "State", width: 122, minWidth: 92, value: (port) => port.state },
        { key: "processName", label: "App", width: 190, minWidth: 130, value: (port) => port.processName },
        { key: "pid", label: "PID", width: 88, minWidth: 72, align: "right", value: (port) => port.pid, cellClass: () => "font-mono text-(--muted)" },
    ];
    rowKey = (port: PortUsage) => this.portKey(port);

    ports = computed(() => this.scan()?.ports ?? []);
    filteredPorts = computed(() => this.filterPorts());
    selectedPort = computed(() => this.filteredPorts().find((port) => this.portKey(port) === this.selectedKey()) ?? this.filteredPorts()[0]);
    listeningCount = computed(() => this.ports().filter((port) => this.isListening(port)).length);
    establishedCount = computed(() => this.ports().filter((port) => port.state.toLowerCase() === "established").length);
    appCount = computed(() => new Set(this.ports().map((port) => `${port.pid ?? "none"}:${port.processName}`)).size);
    publicCount = computed(() => this.ports().filter((port) => port.localAddress === "All interfaces").length);

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
            });
    }

    private isListening(port: PortUsage): boolean {
        const state = port.state.toLowerCase();
        return state === "listen" || state === "listening" || state === "open";
    }
}
