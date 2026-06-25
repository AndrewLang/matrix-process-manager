import { Component, OnDestroy, OnInit, computed, effect, signal } from "@angular/core";
import { NavigationEnd, Router } from "@angular/router";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { filter } from "rxjs";
import { BackendProcessRow, BackendProcessSnapshot, MetricCard, NavItem, ProcessGroup, ProcessRow, ResourceBar, SystemInfoItem, UpdateFrequency, ViewId } from "./app.models";
import { CommonDialogComponent } from "./components/common-dialog/common-dialog.component";
import { SidebarComponent } from "./components/sidebar/sidebar.component";
import { TitlebarComponent } from "./components/titlebar/titlebar.component";
import { WorkareaComponent } from "./components/workarea/workarea.component";
import { SplitterDirective } from "./directives/splitter.directive";
import { WorkareaStateService } from "./services/workarea-state.service";

@Component({
  selector: "mtx-root",
  imports: [CommonDialogComponent, SidebarComponent, SplitterDirective, TitlebarComponent, WorkareaComponent],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnDestroy, OnInit {
  activeView = signal<ViewId>("dashboard");
  selectedProcess = signal("Google Chrome");
  totalProcesses = signal(142);
  sidebarWidth = signal(200);
  settingsDialogOpen = signal(false);
  private refreshTimer?: ReturnType<typeof setInterval>;
  private snapshotInFlight = false;

  overviewItems: NavItem[] = [
    { id: "dashboard", label: "Dashboard", icon: "bi-speedometer2" },
    { id: "processes", label: "Processes", icon: "bi-list-task" },
    { id: "performance", label: "Performance", icon: "bi-activity" },
    { id: "startup", label: "Startup Apps", icon: "bi-rocket-takeoff" },
    { id: "system", label: "System Info", icon: "bi-info-circle" },
  ];

  toolItems: NavItem[] = [
    { id: "processes", label: "Task Manager", icon: "bi-window-stack" },
    { id: "settings", label: "System Setting", icon: "bi-sliders" },
    { id: "disk", label: "Disk Manager", icon: "bi-device-hdd" },
    { id: "terminal", label: "Terminal", icon: "bi-terminal" },
    { id: "more", label: "...", icon: "bi-three-dots" },
  ];

  metrics = signal<MetricCard[]>([
    { label: "CPU", value: "18%", detail: "2.42 GHz", accent: "blue", path: "55,72 75,36 92,70 112,26 126,64 148,54 164,12 180,58 203,46 224,70" },
    { label: "GPU", value: "34%", detail: "6.2 / 8 GB", accent: "cyan", path: "18,66 42,58 64,62 82,34 101,49 126,28 145,56 168,42 188,60 210,30 226,44" },
    { label: "Memory", value: "56%", detail: "8.9 / 16 GB", accent: "violet", path: "12,52 36,50 60,54 82,49 105,60 128,54 145,32 164,47 184,26 205,36 224,32" },
    { label: "Disk", value: "23%", detail: "234 / 1 TB", accent: "green", path: "15,70 50,64 72,18 88,68 112,58 126,23 145,66 168,60 184,20 204,55 220,34" },
    { label: "Network", value: "12%", detail: "12.4 Mbps", accent: "orange", path: "18,68 58,65 82,66 103,30 126,70 148,61 164,68 184,58 206,69" },
  ]);

  rows = signal<ProcessRow[]>([
    { name: "Google Chrome", publisher: "Google LLC", processGroup: "apps", pid: 14532, status: "Running", cpu: "7.3%", gpu: "0%", memory: "1.23 GB", disk: "15.6 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-browser-chrome", selected: true },
    { name: "Google Chrome", publisher: "Google LLC", processGroup: "apps", pid: 14536, status: "Running", cpu: "1.1%", gpu: "0%", memory: "412.8 MB", disk: "1.4 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-browser-chrome" },
    { name: "Visual Studio Code", publisher: "Microsoft Corporation", processGroup: "apps", pid: 11224, status: "Running", cpu: "3.6%", gpu: "0%", memory: "812.4 MB", disk: "2.1 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-code-square" },
    { name: "Slack", publisher: "Slack Technologies", processGroup: "apps", pid: 22344, status: "Running", cpu: "2.1%", gpu: "0%", memory: "598.7 MB", disk: "1.2 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-hash" },
    { name: "Spotify", publisher: "Spotify AB", processGroup: "apps", pid: 33412, status: "Running", cpu: "1.6%", gpu: "0%", memory: "456.1 MB", disk: "0.8 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-music-note-beamed" },
    { name: "Docker Desktop", publisher: "Docker Inc.", processGroup: "apps", pid: 55678, status: "Running", cpu: "0.9%", gpu: "0%", memory: "284.3 MB", disk: "10.3 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-box-seam" },
    { name: "Windows Explorer", publisher: "Microsoft Corporation", processGroup: "windows", pid: 4780, status: "Running", cpu: "0.8%", gpu: "0%", memory: "210.7 MB", disk: "0.4 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-folder" },
    { name: "Terminal", publisher: "Microsoft Corporation", processGroup: "apps", pid: 9512, status: "Running", cpu: "0.6%", gpu: "0%", memory: "168.9 MB", disk: "0.1 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-terminal" },
    { name: "Notion", publisher: "Notion Labs, Inc.", processGroup: "apps", pid: 61988, status: "Running", cpu: "0.4%", gpu: "0%", memory: "156.3 MB", disk: "0.3 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-journal-text" },
    { name: "Microsoft Teams", publisher: "Microsoft Corporation", processGroup: "apps", pid: 27892, status: "Running", cpu: "0.4%", gpu: "0%", memory: "129.8 MB", disk: "0.2 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-people-fill" },
    { name: "Postman", publisher: "Postman Inc.", processGroup: "apps", pid: 14620, status: "Running", cpu: "0.3%", gpu: "0%", memory: "118.6 MB", disk: "0.1 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-send" },
    { name: "WhatsApp", publisher: "WhatsApp LLC", processGroup: "apps", pid: 16320, status: "Running", cpu: "0.3%", gpu: "0%", memory: "112.4 MB", disk: "0.1 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-chat-dots" },
    { name: "OneDrive", publisher: "Microsoft Corporation", processGroup: "background", pid: 25612, status: "Running", cpu: "0.2%", gpu: "0%", memory: "98.7 MB", disk: "0.1 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-cloud" },
    { name: "Service Host", publisher: "Microsoft Corporation", processGroup: "background", pid: 1376, status: "Running", cpu: "0.2%", gpu: "0%", memory: "86.3 MB", disk: "0 MB/s", network: "0 Mbps", user: "system", iconClass: "bi-gear" },
  ]);

  bars = signal<ResourceBar[]>([
    { label: "CPU", value: "7.3%", width: "38%", accent: "blue" },
    { label: "Memory", value: "1.23 GB", width: "28%", accent: "violet" },
    { label: "Disk Read", value: "15.6 MB/s", width: "20%", accent: "green" },
    { label: "Disk Write", value: "8.2 MB/s", width: "42%", accent: "yellow" },
    { label: "Network Sent", value: "5.4 Mbps", width: "40%", accent: "blue" },
    { label: "Network Receive", value: "4.3 Mbps", width: "31%", accent: "blue" },
  ]);

  activeTitle = computed(() => [...this.overviewItems, ...this.toolItems].find((item) => item.id === this.activeView())?.label ?? "Dashboard");

  constructor(private router: Router, public workareaState: WorkareaStateService) {
    this.router.events.pipe(filter((event): event is NavigationEnd => event instanceof NavigationEnd)).subscribe((event) => {
      const view = event.urlAfterRedirects.replace(/^\//, "").split("/")[0];
      if (this.isViewId(view)) {
        this.activeView.set(view);
      }
    });

    effect(() => {
      this.configurePolling(this.workareaState.updateFrequency());
    });
  }

  ngOnInit(): void {
    getCurrentWindow().setIcon("/assets/app-icon.png").catch(() => undefined);
    this.updateSystemInfo();
    this.refreshSnapshot();
  }

  ngOnDestroy(): void {
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
    }
  }

  openSettingsDialog(): void {
    this.settingsDialogOpen.set(true);
  }

  closeSettingsDialog(): void {
    this.settingsDialogOpen.set(false);
  }

  setUpdateFrequency(frequency: UpdateFrequency): void {
    this.workareaState.setUpdateFrequency(frequency);
  }

  refreshSnapshot(): void {
    if (this.snapshotInFlight) {
      return;
    }

    this.snapshotInFlight = true;
    invoke<BackendProcessSnapshot>("get_process_snapshot")
      .then((snapshot) => {
        this.totalProcesses.set(snapshot.totalProcesses);
        const selectedPid = this.workareaState.selectedPid();
        const rows = snapshot.processes.slice(0, 75).map((row) => this.toProcessRow(row, selectedPid));
        this.rows.set(rows);
        this.updateResourceSummary(rows);
      })
      .catch(() => undefined)
      .finally(() => {
        this.snapshotInFlight = false;
      });
  }

  setView(view: ViewId): void {
    this.activeView.set(view);
    this.router.navigate([view]);
  }

  selectProcess(row: ProcessRow): void {
    this.selectedProcess.set(row.name);
  }

  setSidebarWidth(width: number): void {
    this.sidebarWidth.set(width);
  }

  startDrag(event: MouseEvent): void {
    if (event.button !== 0) {
      return;
    }

    getCurrentWindow().startDragging();
  }

  minimize(): void {
    getCurrentWindow().minimize();
  }

  toggleMaximize(): void {
    getCurrentWindow().toggleMaximize();
  }

  close(): void {
    getCurrentWindow().close();
  }

  private toProcessRow(row: BackendProcessRow, selectedPid: number | undefined): ProcessRow {
    return {
      name: row.info.name || `Process ${row.info.pid}`,
      publisher: row.info.publisher || row.info.path || "Unknown publisher",
      processGroup: this.classifyProcess(row),
      iconDataUrl: row.info.iconDataUrl,
      pid: row.info.pid,
      status: row.info.status,
      cpu: `${row.metrics.cpuPercent.toFixed(1)}%`,
      gpu: "0%",
      memory: this.formatBytes(row.metrics.memoryBytes),
      disk: `${this.formatBytes(row.metrics.diskReadBytes + row.metrics.diskWrittenBytes)}/s`,
      network: "0 Mbps",
      user: row.info.user || "system",
      path: row.info.path,
      iconClass: "bi-window",
      selected: row.info.pid === selectedPid,
    };
  }

  private formatBytes(bytes: number): string {
    if (bytes >= 1024 * 1024 * 1024) {
      return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
    }

    if (bytes >= 1024 * 1024) {
      return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    }

    if (bytes >= 1024) {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }

    return `${bytes} B`;
  }

  private classifyProcess(row: BackendProcessRow): ProcessGroup {
    const name = row.info.name.toLowerCase();
    const publisher = row.info.publisher.toLowerCase();
    const user = row.info.user.toLowerCase();

    if (publisher.includes("microsoft") && /windows|explorer|dwm|shell|search|start|runtime/.test(name)) {
      return "windows";
    }

    if (user === "system" || /service|host|daemon|helper|agent|updater|runtime|broker/.test(name)) {
      return "background";
    }

    return "apps";
  }

  private isViewId(view: string): view is ViewId {
    return [...this.overviewItems, ...this.toolItems].some((item) => item.id === view);
  }

  private configurePolling(frequency: UpdateFrequency): void {
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
      this.refreshTimer = undefined;
    }

    if (frequency === "paused") {
      return;
    }

    const interval = frequency === "high" ? 1000 : frequency === "normal" ? 2000 : 5000;
    this.refreshTimer = setInterval(() => this.refreshSnapshot(), interval);
    this.refreshSnapshot();
  }

  private updateResourceSummary(rows: ProcessRow[]): void {
    const cpu = Math.min(100, rows.reduce((total, row) => total + Number.parseFloat(row.cpu), 0));
    const diskBytes = rows.reduce((total, row) => total + this.parseBytesPerSecond(row.disk), 0);
    const memoryBytes = rows.reduce((total, row) => total + this.parseBytes(row.memory), 0);
    const memoryPercent = Math.min(100, memoryBytes / (16 * 1024 * 1024 * 1024) * 100);
    const diskPercent = Math.min(100, diskBytes / (100 * 1024 * 1024) * 100);
    const metrics: MetricCard[] = [
      { label: "CPU", value: `${cpu.toFixed(0)}%`, detail: `${rows.length} visible processes`, accent: "blue", path: this.sparklinePath(cpu) },
      { label: "GPU", value: "0%", detail: "GPU sampling unavailable", accent: "cyan", path: this.sparklinePath(0) },
      { label: "Memory", value: `${memoryPercent.toFixed(0)}%`, detail: this.formatBytes(memoryBytes), accent: "violet", path: this.sparklinePath(memoryPercent) },
      { label: "Disk", value: `${diskPercent.toFixed(0)}%`, detail: `${this.formatBytes(diskBytes)}/s`, accent: "green", path: this.sparklinePath(diskPercent) },
      { label: "Network", value: "0%", detail: "Network sampling unavailable", accent: "orange", path: this.sparklinePath(0) },
    ];
    this.metrics.set(metrics);
    this.bars.set(metrics.map((metric) => ({ label: metric.label, value: metric.detail, width: metric.value, accent: metric.accent })));
    this.updateSystemInfo();
  }

  private updateSystemInfo(): void {
    const metrics = this.metrics();
    const rows = this.rows();
    const info: SystemInfoItem[] = [
      { label: "Platform", value: navigator.platform || "Unknown" },
      { label: "Logical processors", value: navigator.hardwareConcurrency?.toString() ?? "Unknown" },
      { label: "Device memory", value: `${(navigator as Navigator & { deviceMemory?: number }).deviceMemory ?? "Unknown"} GB` },
      { label: "Visible processes", value: rows.length.toString() },
      { label: "Total processes", value: this.totalProcesses().toString() },
      { label: "CPU", value: metrics.find((metric) => metric.label === "CPU")?.value ?? "0%" },
      { label: "Memory", value: metrics.find((metric) => metric.label === "Memory")?.detail ?? "0 B" },
      { label: "Disk throughput", value: metrics.find((metric) => metric.label === "Disk")?.detail ?? "0 B/s" },
      { label: "Update frequency", value: this.workareaState.updateFrequency() },
    ];
    this.workareaState.systemInfo.set(info);
  }

  private sparklinePath(value: number): string {
    return Array.from({ length: 10 }, (_, index) => `${18 + index * 23},${76 - Math.max(0, Math.min(100, value + Math.sin(index) * 8)) * 0.62}`).join(" ");
  }

  private parseBytesPerSecond(value: string): number {
    return this.parseBytes(value.replace(/\/s$/, ""));
  }

  private parseBytes(value: string): number {
    const amount = Number.parseFloat(value) || 0;
    if (value.includes("GB")) {
      return amount * 1024 * 1024 * 1024;
    }

    if (value.includes("MB")) {
      return amount * 1024 * 1024;
    }

    if (value.includes("KB")) {
      return amount * 1024;
    }

    return amount;
  }
}
