import { Component, OnInit, computed, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { BackendProcessRow, BackendProcessSnapshot, MetricCard, NavItem, ProcessRow, ResourceBar, ViewId } from "./app.models";
import { SidebarComponent } from "./components/sidebar/sidebar.component";
import { TitlebarComponent } from "./components/titlebar/titlebar.component";
import { WorkareaComponent } from "./components/workarea/workarea.component";

@Component({
  selector: "app-root",
  imports: [SidebarComponent, TitlebarComponent, WorkareaComponent],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnInit {
  activeView = signal<ViewId>("dashboard");
  selectedProcess = signal("Google Chrome");
  totalProcesses = signal(142);

  overviewItems: NavItem[] = [
    { id: "dashboard", label: "Dashboard", icon: "bi-speedometer2" },
    { id: "processes", label: "Processes", icon: "bi-list-task" },
    { id: "performance", label: "Performance", icon: "bi-activity" },
    { id: "startup", label: "Startup Apps", icon: "bi-rocket-takeoff" },
    { id: "users", label: "Users", icon: "bi-people" },
    { id: "services", label: "Services", icon: "bi-gear" },
    { id: "system", label: "System Info", icon: "bi-info-circle" },
  ];

  toolItems: NavItem[] = [
    { id: "processes", label: "Kill Process", icon: "bi-x-octagon" },
    { id: "performance", label: "Resource Monitor", icon: "bi-cpu" },
    { id: "logs", label: "Logs", icon: "bi-file-earmark-text" },
  ];

  metrics: MetricCard[] = [
    { label: "CPU", value: "18%", detail: "2.42 GHz", accent: "blue", path: "55,72 75,36 92,70 112,26 126,64 148,54 164,12 180,58 203,46 224,70" },
    { label: "GPU", value: "34%", detail: "6.2 / 8 GB", accent: "cyan", path: "18,66 42,58 64,62 82,34 101,49 126,28 145,56 168,42 188,60 210,30 226,44" },
    { label: "Memory", value: "56%", detail: "8.9 / 16 GB", accent: "violet", path: "12,52 36,50 60,54 82,49 105,60 128,54 145,32 164,47 184,26 205,36 224,32" },
    { label: "Disk", value: "23%", detail: "234 / 1 TB", accent: "green", path: "15,70 50,64 72,18 88,68 112,58 126,23 145,66 168,60 184,20 204,55 220,34" },
    { label: "Network", value: "12%", detail: "12.4 Mbps", accent: "orange", path: "18,68 58,65 82,66 103,30 126,70 148,61 164,68 184,58 206,69" },
  ];

  rows = signal<ProcessRow[]>([
    { name: "Google Chrome", publisher: "Google LLC", pid: 14532, status: "Running", cpu: "7.3%", memory: "1.23 GB", disk: "15.6 MB/s", network: "5.4 Mbps", user: "john", iconClass: "bi-browser-chrome", selected: true },
    { name: "Visual Studio Code", publisher: "Microsoft Corporation", pid: 11224, status: "Running", cpu: "3.6%", memory: "812.4 MB", disk: "2.1 MB/s", network: "1.2 Mbps", user: "john", iconClass: "bi-code-square" },
    { name: "Slack", publisher: "Slack Technologies", pid: 22344, status: "Running", cpu: "2.1%", memory: "598.7 MB", disk: "1.2 MB/s", network: "0.6 Mbps", user: "john", iconClass: "bi-hash" },
    { name: "Spotify", publisher: "Spotify AB", pid: 33412, status: "Running", cpu: "1.6%", memory: "456.1 MB", disk: "0.8 MB/s", network: "0.3 Mbps", user: "john", iconClass: "bi-music-note-beamed" },
    { name: "Finder", publisher: "Apple Inc.", pid: 764, status: "Running", cpu: "1.2%", memory: "302.5 MB", disk: "0.2 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-folder2-open" },
    { name: "Docker Desktop", publisher: "Docker Inc.", pid: 55678, status: "Running", cpu: "0.9%", memory: "284.3 MB", disk: "10.3 MB/s", network: "0.2 Mbps", user: "john", iconClass: "bi-box-seam" },
    { name: "Windows Explorer", publisher: "Microsoft Corporation", pid: 4780, status: "Running", cpu: "0.8%", memory: "210.7 MB", disk: "0.4 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-folder" },
    { name: "Terminal", publisher: "Apple Inc.", pid: 9512, status: "Running", cpu: "0.6%", memory: "168.9 MB", disk: "0.1 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-terminal" },
    { name: "Notion", publisher: "Notion Labs, Inc.", pid: 61988, status: "Running", cpu: "0.4%", memory: "156.3 MB", disk: "0.3 MB/s", network: "0.1 Mbps", user: "john", iconClass: "bi-journal-text" },
    { name: "Microsoft Teams", publisher: "Microsoft Corporation", pid: 27892, status: "Running", cpu: "0.4%", memory: "129.8 MB", disk: "0.2 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-people-fill" },
    { name: "Postman", publisher: "Postman Inc.", pid: 14620, status: "Running", cpu: "0.3%", memory: "118.6 MB", disk: "0.1 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-send" },
    { name: "WhatsApp", publisher: "WhatsApp LLC", pid: 16320, status: "Running", cpu: "0.3%", memory: "112.4 MB", disk: "0.1 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-chat-dots" },
    { name: "OneDrive", publisher: "Microsoft Corporation", pid: 25612, status: "Running", cpu: "0.2%", memory: "98.7 MB", disk: "0.1 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-cloud" },
    { name: "Activity Monitor", publisher: "Apple Inc.", pid: 1376, status: "Running", cpu: "0.2%", memory: "86.3 MB", disk: "0 MB/s", network: "0 Mbps", user: "john", iconClass: "bi-graph-up" },
  ]);

  bars: ResourceBar[] = [
    { label: "CPU", value: "7.3%", width: "38%", accent: "blue" },
    { label: "Memory", value: "1.23 GB", width: "28%", accent: "violet" },
    { label: "Disk Read", value: "15.6 MB/s", width: "20%", accent: "green" },
    { label: "Disk Write", value: "8.2 MB/s", width: "42%", accent: "yellow" },
    { label: "Network Sent", value: "5.4 Mbps", width: "40%", accent: "blue" },
    { label: "Network Receive", value: "4.3 Mbps", width: "31%", accent: "blue" },
  ];

  activeTitle = computed(() => this.overviewItems.find((item) => item.id === this.activeView())?.label ?? "Dashboard");

  ngOnInit(): void {
    invoke<BackendProcessSnapshot>("get_process_snapshot")
      .then((snapshot) => {
        this.totalProcesses.set(snapshot.totalProcesses);
        this.rows.set(snapshot.processes.slice(0, 50).map((row, index) => this.toProcessRow(row, index)));
      })
      .catch(() => undefined);
  }

  setView(view: ViewId): void {
    this.activeView.set(view);
  }

  selectProcess(row: ProcessRow): void {
    this.selectedProcess.set(row.name);
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

  private toProcessRow(row: BackendProcessRow, index: number): ProcessRow {
    return {
      name: row.info.name || `Process ${row.info.pid}`,
      publisher: row.info.publisher || row.info.path || "Unknown publisher",
      pid: row.info.pid,
      status: row.info.status,
      cpu: `${row.metrics.cpuPercent.toFixed(1)}%`,
      memory: this.formatBytes(row.metrics.memoryBytes),
      disk: `${this.formatBytes(row.metrics.diskReadBytes + row.metrics.diskWrittenBytes)}/s`,
      network: "0 Mbps",
      user: row.info.user || "system",
      iconClass: "bi-window",
      selected: index === 0,
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
}
