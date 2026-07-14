import { Component, HostListener, OnDestroy, OnInit, computed, effect, signal } from "@angular/core";
import { NavigationEnd, Router } from "@angular/router";
import { invoke } from "@tauri-apps/api/core";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { filter } from "rxjs";
import { BackendCpuInfo, BackendProcessRow, BackendProcessSnapshot, BackendWindowsInfo, DockerAvailability, MetricCard, NativeToolId, NavItem, ProcessGroup, ProcessRow, ResourceBar, ResourceSample, SystemInfoItem, UpdateFrequency, ViewId } from "./app.models";
import { CommonDialogComponent } from "./components/common-dialog/common-dialog.component";
import { SidebarComponent } from "./components/sidebar/sidebar.component";
import { TitlebarComponent } from "./components/titlebar/titlebar.component";
import { WorkareaComponent } from "./components/workarea/workarea.component";
import { SplitterDirective } from "./directives/splitter.directive";
import { ProcessSnapshotWorkerRequest, ProcessSnapshotWorkerResponse } from "./process-snapshot.worker";
import { WorkareaStateService } from "./services/workarea-state.service";
import { SettingsViewComponent } from "./views/settings-view/settings-view.component";

interface PersistedUiState {
  activeView: ViewId;
  route: string;
  sidebarWidth: number;
  updateFrequency: UpdateFrequency;
}

interface PersistedWindowState {
  x: number;
  y: number;
  width: number;
  height: number;
  maximized: boolean;
}

@Component({
  selector: "mtx-root",
  imports: [CommonDialogComponent, SettingsViewComponent, SidebarComponent, SplitterDirective, TitlebarComponent, WorkareaComponent],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnDestroy, OnInit {
  private readonly uiStateKey = "workstation-console.ui-state";
  private readonly legacyUiStateKey = "matrix-process-manager.ui-state";
  private readonly windowStateKey = "workstation-console.window-state";
  private readonly legacyWindowStateKey = "matrix-process-manager.window-state";
  private readonly persistedUiState = this.loadUiState();
  activeView = signal<ViewId>(this.persistedUiState?.activeView ?? "dashboard");
  selectedProcess = signal("Google Chrome");
  totalProcesses = signal(142);
  sidebarWidth = signal(this.persistedUiState?.sidebarWidth ?? 200);
  workstationName = signal("My Workstation");
  settingsDialogOpen = signal(false);
  dockerInstalled = signal(false);
  private refreshTimer?: ReturnType<typeof setInterval>;
  private uiSaveTimer?: ReturnType<typeof setTimeout>;
  private windowSaveTimer?: ReturnType<typeof setTimeout>;
  private snapshotInFlight = false;
  private refreshPausedUntil = 0;
  private processOrder: number[] = [];
  private metricHistory: ResourceSample[] = [];
  private cpuInfo?: BackendCpuInfo;
  private windowsInfo?: BackendWindowsInfo;
  private processWorker?: Worker;
  private transformRequestId = 0;
  private pendingTransforms = new Map<number, { resolve: (response: ProcessSnapshotWorkerResponse) => void; reject: () => void }>();
  private windowUnlisteners: Array<() => void> = [];
  private restoringWindowState = false;
  private normalWindowState?: PersistedWindowState;

  baseOverviewItems: NavItem[] = [
    { id: "dashboard", label: "Dashboard", icon: "bi-speedometer2" },
    { id: "processes", label: "Processes", icon: "bi-list-task" },
    { id: "performance", label: "Performance", icon: "bi-activity" },
    { id: "storage", label: "Storage", icon: "bi-device-ssd" },
    { id: "ports", label: "Ports", icon: "bi-ethernet" },
    { id: "network-devices", label: "Network", icon: "bi-router" },
    { id: "ssh-keys", label: "SSH Keys", icon: "bi-key" },
    { id: "startup", label: "Startup Apps", icon: "bi-rocket-takeoff" },
    { id: "system", label: "System Info", icon: "bi-info-circle" },
    { id: "command-center", label: "Console (Beta)", icon: "bi-terminal-plus" },
  ];

  toolItems: NavItem[] = [
    { id: "processes", label: "Task Manager", icon: "bi-window-stack", nativeTool: "taskManager" },
    { id: "settings", label: "System Setting", icon: "bi-sliders", nativeTool: "systemSettings" },
    { id: "storage", label: "Disk Manager", icon: "bi-device-hdd", nativeTool: "diskManager" },
    { id: "terminal", label: "Terminal", icon: "bi-terminal", nativeTool: "terminal" },
    { id: "settings", label: "Env Variables", icon: "bi-braces", nativeTool: "envVariables" },
    { id: "settings", label: "Snipping Tool", icon: "bi-scissors", nativeTool: "snippingTool" },
  ];
  overviewItems = computed<NavItem[]>(() => this.dockerInstalled()
    ? [...this.baseOverviewItems.slice(0, 6), { id: "docker", label: "Docker", icon: "bi-box-seam" }, ...this.baseOverviewItems.slice(6)]
    : this.baseOverviewItems);

  enabledToolItems = computed(() => this.toolItems.filter((item) => !item.nativeTool || this.workareaState.appSettings().toolSettings[item.nativeTool]));
  localIpAddress = computed(() => this.findLocalIpAddress());

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

  activeTitle = computed(() => [...this.overviewItems(), ...this.toolItems].find((item) => item.id === this.activeView())?.label ?? "Dashboard");

  constructor(private router: Router, public workareaState: WorkareaStateService) {
    if (this.persistedUiState) {
      this.workareaState.setUpdateFrequency(this.persistedUiState.updateFrequency);
    }

    this.router.events.pipe(filter((event): event is NavigationEnd => event instanceof NavigationEnd)).subscribe((event) => {
      const view = event.urlAfterRedirects.replace(/^\//, "").split("/")[0];
      if (this.isViewId(view)) {
        this.activeView.set(view);
        this.scheduleUiStateSave(event.urlAfterRedirects);
      }
    });

    effect(() => {
      this.configurePolling(this.workareaState.updateFrequency());
    });
  }

  ngOnInit(): void {
    this.refreshWindowIcon();
    this.restoreWindowState().then(() => this.trackWindowState());
    this.restoreRoute();
    this.startProcessWorker();
    this.refreshDockerAvailability();
    this.updateSystemInfo();
    this.refreshSnapshot();
  }

  ngOnDestroy(): void {
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
    }
    if (this.uiSaveTimer) {
      clearTimeout(this.uiSaveTimer);
    }
    if (this.windowSaveTimer) {
      clearTimeout(this.windowSaveTimer);
    }
    this.saveUiState(this.router.url);
    this.saveWindowState();
    for (const unlisten of this.windowUnlisteners) {
      unlisten();
    }
    this.processWorker?.terminate();
  }

  openSettingsDialog(): void {
    this.settingsDialogOpen.set(true);
  }

  closeSettingsDialog(): void {
    this.settingsDialogOpen.set(false);
  }

  setUpdateFrequency(frequency: UpdateFrequency): void {
    this.workareaState.setUpdateFrequency(frequency);
    this.scheduleUiStateSave(this.router.url);
  }

  @HostListener("window:focus")
  refreshWindowIconFromWindowFocus(): void {
    this.refreshWindowIcon();
  }

  @HostListener("document:visibilitychange")
  refreshWindowIconFromVisibilityChange(): void {
    if (document.visibilityState === "visible") {
      this.refreshWindowIcon();
    }
  }

  resetSettings(): void {
    this.workareaState.resetAppSettings();
    invoke<void>("set_start_with_windows", { enabled: false }).catch(() => undefined);
  }

  refreshDockerAvailability(): void {
    invoke<DockerAvailability>("get_docker_availability", { dockerHost: "" })
      .then((availability) => this.dockerInstalled.set(availability.installed))
      .catch(() => this.dockerInstalled.set(false));
  }

  refreshSnapshot(): void {
    if (this.snapshotInFlight || this.isRefreshPaused()) {
      return;
    }

    this.snapshotInFlight = true;
    invoke<BackendProcessSnapshot>("get_process_snapshot")
      .then(async (snapshot) => {
        if (this.isRefreshPaused()) {
          return;
        }

        this.totalProcesses.set(snapshot.totalProcesses);
        this.cpuInfo = snapshot.cpuInfo;
        this.windowsInfo = snapshot.windowsInfo;
        this.workstationName.set(snapshot.windowsInfo.deviceName || "My Workstation");
        this.workareaState.setMemoryInfo(snapshot.memoryInfo);
        this.workareaState.setGpuAdapters(snapshot.gpuAdapters);
        this.workareaState.setDiskDrives(snapshot.diskDrives);
        this.workareaState.setNetworkAdapters(snapshot.networkAdapters);
        const selectedPid = this.workareaState.selectedPid();
        const transformed = await this.transformProcesses(snapshot.processes, selectedPid);
        if (this.isRefreshPaused()) {
          return;
        }

        this.processOrder = transformed.processOrder;
        this.rows.set(transformed.rows);
        this.updateResourceSummary(transformed.rows, transformed.diskBytes, snapshot.totalCpuPercent, snapshot.totalGpuPercent, snapshot.totalDiskPercent, snapshot.totalNetworkPercent, snapshot.usedMemoryBytes, snapshot.totalMemoryBytes);
      })
      .catch(() => undefined)
      .finally(() => {
        this.snapshotInFlight = false;
      });
  }

  setView(view: ViewId): void {
    this.activeView.set(view);
    this.scheduleUiStateSave(`/${view}`);
    this.router.navigate([view]);
  }

  openTool(item: NavItem): void {
    if (!item.nativeTool) {
      this.setView(item.id);
      return;
    }

    if (!this.workareaState.appSettings().toolSettings[item.nativeTool]) {
      return;
    }

    invoke<void>("open_native_tool", { toolId: item.nativeTool satisfies NativeToolId }).catch(() => undefined);
  }

  selectProcess(row: ProcessRow): void {
    this.selectedProcess.set(row.name);
  }

  setSidebarWidth(width: number): void {
    this.sidebarWidth.set(width);
    this.scheduleUiStateSave(this.router.url);
  }

  pauseRefreshForDrag(event: MouseEvent): void {
    if (event.button !== 0) {
      return;
    }

    this.refreshPausedUntil = Date.now() + 1500;
  }

  private isRefreshPaused(): boolean {
    return Date.now() < this.refreshPausedUntil;
  }

  minimize(): void {
    const appWindow = getCurrentWindow();
    if (this.workareaState.appSettings().minimizeToTray) {
      this.saveWindowState().finally(() => appWindow.hide());
      return;
    }

    appWindow.minimize();
  }

  toggleMaximize(): void {
    getCurrentWindow().toggleMaximize();
  }

  async close(): Promise<void> {
    this.saveUiState(this.router.url);
    await this.saveWindowState();
    this.processWorker?.terminate();
    const appWindow = getCurrentWindow();
    appWindow.destroy().catch(() => appWindow.close());
  }

  private restoreRoute(): void {
    const route = this.persistedUiState?.route;
    if (!route || this.router.url !== "/" && this.router.url !== "/dashboard") {
      return;
    }

    queueMicrotask(() => {
      this.router.navigateByUrl(route).catch(() => undefined);
    });
  }

  private async restoreWindowState(): Promise<void> {
    const state = this.loadWindowState();
    const appWindow = getCurrentWindow();
    this.restoringWindowState = true;
    this.normalWindowState = state;

    try {
      if (state) {
        await appWindow.setPosition(new PhysicalPosition(state.x, state.y));
        await appWindow.setSize(new PhysicalSize(state.width, state.height));
      }

      await appWindow.show();

      if (state) {
        await appWindow.setSize(new PhysicalSize(state.width, state.height));
        await appWindow.setPosition(new PhysicalPosition(state.x, state.y));
        if (state.maximized) {
          await appWindow.maximize();
        }
      }
    } catch {
      appWindow.show().catch(() => undefined);
    } finally {
      setTimeout(() => {
        this.restoringWindowState = false;
      }, 300);
    }
  }

  private trackWindowState(): void {
    const appWindow = getCurrentWindow();
    appWindow.onMoved(() => this.scheduleWindowStateSave()).then((unlisten) => this.windowUnlisteners.push(unlisten)).catch(() => undefined);
    appWindow.onResized(() => this.scheduleWindowStateSave()).then((unlisten) => this.windowUnlisteners.push(unlisten)).catch(() => undefined);
    appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        this.refreshWindowIcon();
      }

      if (focused && this.workareaState.appSettings().minimizeToTray) {
        setTimeout(() => this.restoreNormalWindowBounds(), 0);
      }
    }).then((unlisten) => this.windowUnlisteners.push(unlisten)).catch(() => undefined);
    appWindow.onCloseRequested(() => this.saveWindowState()).then((unlisten) => this.windowUnlisteners.push(unlisten)).catch(() => undefined);
  }

  private scheduleUiStateSave(route: string): void {
    if (this.uiSaveTimer) {
      clearTimeout(this.uiSaveTimer);
    }

    this.uiSaveTimer = setTimeout(() => this.saveUiState(route), 120);
  }

  private refreshWindowIcon(): void {
    invoke<void>("refresh_window_icon").catch(() => undefined);
  }

  private saveUiState(route: string): void {
    const state: PersistedUiState = {
      activeView: this.activeView(),
      route: this.normalizeRoute(route, this.activeView()),
      sidebarWidth: this.sidebarWidth(),
      updateFrequency: this.workareaState.updateFrequency(),
    };
    this.writeJson(this.uiStateKey, state);
  }

  private scheduleWindowStateSave(): void {
    if (this.restoringWindowState) {
      return;
    }

    if (this.windowSaveTimer) {
      clearTimeout(this.windowSaveTimer);
    }

    this.windowSaveTimer = setTimeout(() => this.saveWindowState(), 250);
  }

  private saveWindowState(): Promise<void> {
    if (this.restoringWindowState) {
      return Promise.resolve();
    }

    const appWindow = getCurrentWindow();
    return Promise.all([appWindow.outerPosition(), appWindow.outerSize(), appWindow.isMaximized(), appWindow.isMinimized(), appWindow.isVisible()])
      .then(([position, size, maximized, minimized, visible]) => {
        if (maximized || minimized || !visible) {
          if (this.normalWindowState) {
            this.writeJson(this.windowStateKey, { ...this.normalWindowState, maximized } satisfies PersistedWindowState);
          }
          return;
        }

        const next = {
          x: position.x,
          y: position.y,
          width: size.width,
          height: size.height,
          maximized,
        } satisfies PersistedWindowState;
        this.normalWindowState = next;
        this.writeJson(this.windowStateKey, next);
      })
      .catch(() => undefined);
  }

  private async restoreNormalWindowBounds(): Promise<void> {
    const state = this.normalWindowState ?? this.loadWindowState();
    if (!state) {
      return;
    }

    const appWindow = getCurrentWindow();
    this.restoringWindowState = true;
    try {
      const maximized = await appWindow.isMaximized();
      if (!maximized) {
        await appWindow.setSize(new PhysicalSize(state.width, state.height));
        await appWindow.setPosition(new PhysicalPosition(state.x, state.y));
      }
    } finally {
      setTimeout(() => {
        this.restoringWindowState = false;
      }, 300);
    }
  }

  private loadUiState(): PersistedUiState | undefined {
    const state = this.readJson<Partial<PersistedUiState>>(this.uiStateKey) ?? this.readJson<Partial<PersistedUiState>>(this.legacyUiStateKey);
    if (!state || !this.isPersistedViewId(state.activeView) || !this.isUpdateFrequency(state.updateFrequency)) {
      return undefined;
    }

    const activeView = state.activeView === "disk" ? "storage" : state.activeView;

    return {
      activeView,
      route: this.normalizeRoute(state.route, activeView),
      sidebarWidth: this.clampSidebarWidth(state.sidebarWidth),
      updateFrequency: state.updateFrequency,
    };
  }

  private loadWindowState(): PersistedWindowState | undefined {
    const state = this.readJson<Partial<PersistedWindowState>>(this.windowStateKey) ?? this.readJson<Partial<PersistedWindowState>>(this.legacyWindowStateKey);
    if (!state || !Number.isFinite(state.x) || !Number.isFinite(state.y) || !Number.isFinite(state.width) || !Number.isFinite(state.height)) {
      return undefined;
    }

    const x = state.x as number;
    const y = state.y as number;
    const width = state.width as number;
    const height = state.height as number;

    return {
      x: Math.round(x),
      y: Math.round(y),
      width: Math.max(980, Math.round(width)),
      height: Math.max(640, Math.round(height)),
      maximized: state.maximized === true,
    };
  }

  private normalizeRoute(route: unknown, fallbackView: ViewId): string {
    if (typeof route !== "string") {
      return `/${fallbackView}`;
    }

    const path = route.startsWith("/") ? route : `/${route}`;
    const firstSegment = path.replace(/^\//, "").split("/")[0];
    if (firstSegment === "disk") {
      return "/storage";
    }

    return this.isPersistedViewId(firstSegment) ? path : `/${fallbackView}`;
  }

  private clampSidebarWidth(width: unknown): number {
    return Math.max(120, Math.min(280, typeof width === "number" && Number.isFinite(width) ? width : 200));
  }

  private readJson<T>(key: string): T | undefined {
    try {
      const value = localStorage.getItem(key);
      return value ? JSON.parse(value) as T : undefined;
    } catch {
      return undefined;
    }
  }

  private writeJson(key: string, value: unknown): void {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch {
      return;
    }
  }

  private isPersistedViewId(value: unknown): value is ViewId {
    return value === "dashboard" || value === "processes" || value === "performance" || value === "startup" || value === "system" || value === "command-center" || value === "settings" || value === "storage" || value === "ports" || value === "ssh-keys" || value === "docker" || value === "disk" || value === "terminal" || value === "more";
  }

  private isUpdateFrequency(value: unknown): value is UpdateFrequency {
    return value === "high" || value === "normal" || value === "low" || value === "paused";
  }

  private startProcessWorker(): void {
    if (typeof Worker === "undefined") {
      return;
    }

    this.processWorker = new Worker(new URL("./process-snapshot.worker", import.meta.url), { type: "module" });
    this.processWorker.onmessage = (event: MessageEvent<ProcessSnapshotWorkerResponse>) => {
      const pending = this.pendingTransforms.get(event.data.requestId);
      if (!pending) {
        return;
      }

      this.pendingTransforms.delete(event.data.requestId);
      pending.resolve(event.data);
    };
    this.processWorker.onerror = () => {
      for (const pending of this.pendingTransforms.values()) {
        pending.reject();
      }
      this.pendingTransforms.clear();
      this.processWorker?.terminate();
      this.processWorker = undefined;
    };
  }

  private transformProcesses(processes: BackendProcessRow[], selectedPid: number | undefined): Promise<ProcessSnapshotWorkerResponse> {
    if (!this.processWorker) {
      return Promise.resolve(this.transformProcessesInThread(processes, selectedPid));
    }

    const requestId = ++this.transformRequestId;
    const request: ProcessSnapshotWorkerRequest = {
      requestId,
      processes,
      selectedPid,
      processOrder: this.processOrder,
    };

    return new Promise<ProcessSnapshotWorkerResponse>((resolve, reject) => {
      this.pendingTransforms.set(requestId, { resolve, reject });
      this.processWorker?.postMessage(request);
    }).catch(() => this.transformProcessesInThread(processes, selectedPid));
  }

  private transformProcessesInThread(processes: BackendProcessRow[], selectedPid: number | undefined): ProcessSnapshotWorkerResponse {
    const rows = this.stabilizeProcessOrder(processes).map((row) => this.toProcessRow(row, selectedPid));
    return {
      requestId: 0,
      rows,
      processOrder: this.processOrder,
      diskBytes: processes.reduce((total, row) => total + row.metrics.diskReadBytes + row.metrics.diskWrittenBytes, 0),
    };
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
      gpu: `${row.metrics.gpuPercent.toFixed(1)}%`,
      memory: this.formatBytes(row.metrics.memoryBytes),
      disk: `${this.formatBytes(row.metrics.diskReadBytes + row.metrics.diskWrittenBytes)}/s`,
      network: "0 Mbps",
      user: row.info.user || "system",
      path: row.info.path,
      iconClass: "bi-window",
      selected: row.info.pid === selectedPid,
    };
  }

  private stabilizeProcessOrder(processes: BackendProcessRow[]): BackendProcessRow[] {
    const currentPids = new Set(processes.map((process) => process.info.pid));
    const knownPids = new Set(this.processOrder);
    this.processOrder = this.processOrder.filter((pid) => currentPids.has(pid));

    for (const process of processes) {
      if (!knownPids.has(process.info.pid)) {
        this.processOrder.push(process.info.pid);
      }
    }

    const order = new Map(this.processOrder.map((pid, index) => [pid, index]));
    return [...processes].sort((left, right) => (order.get(left.info.pid) ?? Number.MAX_SAFE_INTEGER) - (order.get(right.info.pid) ?? Number.MAX_SAFE_INTEGER));
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

  private findLocalIpAddress(): string {
    return this.workareaState.networkAdapters()
      .flatMap((adapter) => adapter.ipv4Addresses)
      .find((address) => this.isUsableLocalIpAddress(address)) ?? "Unavailable";
  }

  private isUsableLocalIpAddress(address: string): boolean {
    return Boolean(address)
      && address !== "127.0.0.1"
      && !address.startsWith("169.254.")
      && !address.startsWith("0.");
  }

  private classifyProcess(row: BackendProcessRow): ProcessGroup {
    const name = row.info.name.toLowerCase();
    const publisher = row.info.publisher.toLowerCase();
    const path = row.info.path.toLowerCase();
    const user = row.info.user.toLowerCase();

    if (row.info.hasVisibleWindow) {
      return "apps";
    }

    if ((publisher.includes("microsoft") || path.includes("\\windows\\")) && /windows|explorer|dwm|shell|search|start|runtime|system|registry|font|spool|audio|defender/.test(name)) {
      return "windows";
    }

    if (user === "system" || /service|host|daemon|helper|agent|updater|runtime|broker|crashpad|utility|worker|background/.test(name)) {
      return "background";
    }

    return "background";
  }

  private isViewId(view: string): view is ViewId {
    return [...this.overviewItems(), ...this.toolItems].some((item) => item.id === view);
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

  private updateResourceSummary(rows: ProcessRow[], diskBytes: number, totalCpuPercent: number, totalGpuPercent: number, totalDiskPercent: number, totalNetworkPercent: number, usedMemoryBytes: number, totalMemoryBytes: number): void {
    const cpu = Math.max(0, Math.min(100, totalCpuPercent));
    const gpu = Math.max(0, Math.min(100, totalGpuPercent));
    const disk = Math.max(0, Math.min(100, totalDiskPercent));
    const network = Math.max(0, Math.min(100, totalNetworkPercent));
    const memoryBytes = Math.max(0, usedMemoryBytes);
    const memoryPercent = totalMemoryBytes > 0 ? Math.min(100, memoryBytes / totalMemoryBytes * 100) : 0;
    const sample: ResourceSample = { cpu, gpu, memory: memoryPercent, disk, network };
    this.metricHistory = [...this.metricHistory.slice(-59), sample];
    const metrics: MetricCard[] = [
      { label: "CPU", value: `${cpu.toFixed(0)}%`, detail: "Total CPU usage", accent: "blue", path: this.historyPath("cpu") },
      { label: "GPU", value: `${gpu.toFixed(0)}%`, detail: "GPU engine utilization", accent: "cyan", path: this.historyPath("gpu") },
      { label: "Memory", value: `${memoryPercent.toFixed(0)}%`, detail: this.formatBytes(memoryBytes), accent: "violet", path: this.historyPath("memory") },
      { label: "Disk", value: `${disk.toFixed(0)}%`, detail: `${this.formatBytes(diskBytes)}/s`, accent: "green", path: this.historyPath("disk") },
      { label: "Network", value: `${network.toFixed(0)}%`, detail: `${this.formatBytes(this.workareaState.networkAdapters().reduce((total, adapter) => total + adapter.receiveBytesPerSec + adapter.sendBytesPerSec, 0))}/s`, accent: "orange", path: this.historyPath("network") },
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
      { label: "Device name", value: this.windowsInfo?.deviceName || "Unknown" },
      { label: "System product name", value: this.windowsInfo?.model || "Unknown" },
      { label: "Manufacturer", value: this.windowsInfo?.manufacturer || "Unknown" },
      { label: "System type", value: this.windowsInfo?.systemType || "Unknown" },
      { label: "Device ID", value: this.windowsInfo?.deviceId || "Unavailable" },
      { label: "Product ID", value: this.windowsInfo?.productId || "Unavailable" },
      { label: "OS edition", value: this.windowsInfo?.osEdition || "Unknown" },
      { label: "OS version", value: this.windowsInfo?.osVersion || "Unknown" },
      { label: "Installed on", value: this.windowsInfo?.installedOn || "Unavailable" },
      { label: "OS build", value: this.windowsInfo?.osBuild || "Unavailable" },
      { label: "System firmware", value: this.windowsInfo?.experience || "Unavailable" },
      { label: "Logical processors", value: navigator.hardwareConcurrency?.toString() ?? "Unknown" },
      { label: "Device memory", value: `${(navigator as Navigator & { deviceMemory?: number }).deviceMemory ?? "Unknown"} GB` },
      { label: "Visible processes", value: rows.length.toString() },
      { label: "Total processes", value: this.totalProcesses().toString() },
      { label: "CPU", value: metrics.find((metric) => metric.label === "CPU")?.value ?? "0%" },
      { label: "CPU model", value: this.cpuInfo?.model || "Unknown" },
      { label: "CPU current speed", value: this.formatMegahertz(this.cpuInfo?.currentSpeedMhz ?? 0) },
      { label: "CPU base speed", value: this.formatMegahertz(this.cpuInfo?.baseSpeedMhz ?? 0) },
      { label: "Sockets", value: (this.cpuInfo?.sockets ?? 1).toString() },
      { label: "Cores", value: (this.cpuInfo?.cores ?? navigator.hardwareConcurrency ?? 0).toString() },
      { label: "CPU logical processors", value: (this.cpuInfo?.logicalProcessors ?? navigator.hardwareConcurrency ?? 0).toString() },
      { label: "Threads", value: this.cpuInfo?.totalThreads != null ? this.cpuInfo.totalThreads.toString() : "Unavailable" },
      { label: "Handles", value: this.cpuInfo?.totalHandles != null ? this.cpuInfo.totalHandles.toString() : "Unavailable" },
      { label: "Up time", value: this.formatDuration(this.cpuInfo?.uptimeSeconds ?? 0) },
      { label: "Virtualization", value: this.cpuInfo?.virtualization ?? "Unavailable" },
      { label: "L1 cache", value: this.cpuInfo?.l1CacheBytes ? this.formatBytes(this.cpuInfo.l1CacheBytes) : "Unavailable" },
      { label: "L2 cache", value: this.cpuInfo?.l2CacheBytes ? this.formatBytes(this.cpuInfo.l2CacheBytes) : "Unavailable" },
      { label: "L3 cache", value: this.cpuInfo?.l3CacheBytes ? this.formatBytes(this.cpuInfo.l3CacheBytes) : "Unavailable" },
      { label: "Memory", value: metrics.find((metric) => metric.label === "Memory")?.detail ?? "0 B" },
      { label: "Disk throughput", value: metrics.find((metric) => metric.label === "Disk")?.detail ?? "0 B/s" },
      { label: "Update frequency", value: this.workareaState.updateFrequency() },
    ];
    this.workareaState.systemInfo.set(info);
  }

  private historyPath(metric: keyof ResourceSample): string {
    const history = this.metricHistory;
    if (history.length === 0) {
      return "18,76 224,76";
    }

    if (history.length === 1) {
      const y = this.historyY(history[0][metric]);
      return `18,${y} 224,${y}`;
    }

    return history.map((sample, index) => {
      const x = 18 + index * (206 / (history.length - 1));
      return `${x.toFixed(1)},${this.historyY(sample[metric])}`;
    }).join(" ");
  }

  private historyY(value: number): string {
    return (76 - Math.max(0, Math.min(100, value)) * 0.62).toFixed(1);
  }

  private formatMegahertz(value: number): string {
    if (value <= 0) {
      return "Unavailable";
    }

    return `${(value / 1000).toFixed(2)} GHz`;
  }

  private formatDuration(totalSeconds: number): string {
    const days = Math.floor(totalSeconds / 86400);
    const hours = Math.floor(totalSeconds % 86400 / 3600).toString().padStart(2, "0");
    const minutes = Math.floor(totalSeconds % 3600 / 60).toString().padStart(2, "0");
    const seconds = Math.floor(totalSeconds % 60).toString().padStart(2, "0");
    return `${days}:${hours}:${minutes}:${seconds}`;
  }

}
