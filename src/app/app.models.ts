export type ViewId = "dashboard" | "processes" | "performance" | "startup" | "system" | "command-center" | "settings" | "storage" | "ports" | "network-devices" | "ssh-keys" | "docker" | "disk" | "terminal" | "more";
export type NativeToolId = "taskManager" | "systemSettings" | "diskManager" | "terminal" | "envVariables" | "snippingTool";
export type ProcessGroup = "apps" | "background" | "windows";
export type UpdateFrequency = "high" | "normal" | "low" | "paused";
export type TerminalDefaultShell = "system" | "powerShell" | "cmd" | "zsh" | "bash";
export type TerminalCursorStyle = "block" | "bar" | "underline";
export type TerminalTheme = "matrix" | "midnight" | "slate";
export type IndexingSchedule = "manual" | "startup" | "hourly" | "daily";

export interface TerminalSettings {
    defaultShell: TerminalDefaultShell;
    fontFamily: string;
    fontSize: number;
    cursorStyle: TerminalCursorStyle;
    opacity: number;
    theme: TerminalTheme;
    historySize: number;
    commandIntelligenceEnabled: boolean;
    autocompleteDelayMs: number;
}

export interface IndexingSettings {
    schedule: IndexingSchedule;
}

export interface StorageSettings {
    sqliteLocation: string;
}

export interface AppSettings {
    startWithWindows: boolean;
    minimizeToTray: boolean;
    confirmBeforeKillingProcesses: boolean;
    terminalSettings: TerminalSettings;
    indexingSettings: IndexingSettings;
    storageSettings: StorageSettings;
    toolSettings: Record<NativeToolId, boolean>;
}

export interface NavItem {
    id: ViewId;
    label: string;
    icon: string;
    nativeTool?: NativeToolId;
}

export interface MetricCard {
    label: string;
    value: string;
    detail: string;
    accent: string;
    path: string;
}

export interface ProcessRow {
    name: string;
    publisher: string;
    processGroup?: ProcessGroup;
    iconDataUrl?: string;
    pid: number;
    status: string;
    cpu: string;
    gpu: string;
    memory: string;
    disk: string;
    network: string;
    user: string;
    path?: string;
    iconClass: string;
    selected?: boolean;
}

export interface ResourceBar {
    label: string;
    value: string;
    width: string;
    accent: string;
}

export interface ResourceSample {
    cpu: number;
    gpu: number;
    memory: number;
    disk: number;
    network: number;
}

export interface SystemInfoItem {
    label: string;
    value: string;
}

export interface StartupApp {
    name: string;
    publisher: string;
    iconDataUrl?: string;
    status: string;
    impact: string;
    startupType: string;
    source: string;
    command: string;
    path: string;
    valueName?: string;
    delaySeconds?: number;
}

export interface BackendProcessSnapshot {
    processes: BackendProcessRow[];
    totalProcesses: number;
    totalCpuPercent: number;
    totalGpuPercent: number;
    totalDiskPercent: number;
    totalNetworkPercent: number;
    usedMemoryBytes: number;
    totalMemoryBytes: number;
    cpuInfo: BackendCpuInfo;
    memoryInfo: BackendMemoryInfo;
    gpuAdapters: BackendGpuAdapterUsage[];
    diskDrives: BackendDiskDriveUsage[];
    networkAdapters: BackendNetworkAdapterUsage[];
    windowsInfo: BackendWindowsInfo;
}

export interface BackendWindowsInfo {
    deviceName?: string;
    manufacturer?: string;
    model?: string;
    systemType?: string;
    deviceId?: string;
    productId?: string;
    osEdition?: string;
    osVersion?: string;
    installedOn?: string;
    osBuild?: string;
    experience?: string;
}

export interface BackendNetworkAdapterUsage {
    name: string;
    adapterIndex: number;
    utilizationPercent: number;
    receiveBytesPerSec: number;
    sendBytesPerSec: number;
    linkSpeedBitsPerSec?: number;
    connectionName?: string;
    macAddress?: string;
    adapterType?: string;
    ipv4Addresses: string[];
    ipv6Addresses: string[];
}

export interface BackendDiskDriveUsage {
    name: string;
    labels: string[];
    diskIndex: number;
    activeTimePercent: number;
    averageResponseTimeMs: number;
    readBytesPerSec: number;
    writeBytesPerSec: number;
    capacityBytes?: number;
    formattedBytes?: number;
    systemDisk?: boolean;
    pageFile?: boolean;
    diskType?: string;
}

export interface DiskVolumeUsage {
    label: string;
    name: string;
    totalBytes: number;
    freeBytes: number;
    systemDrive: boolean;
}

export interface DiskCleanupTarget {
    id: string;
    name: string;
    path: string;
    description: string;
    bytes: number;
    exists: boolean;
}

export interface DiskUsageInsight {
    id: string;
    name: string;
    path: string;
    category: string;
    description: string;
    safeToClean: boolean;
    safety: string;
    bytes: number;
    exists: boolean;
}

export interface DiskCleanupScan {
    volumes: DiskVolumeUsage[];
    targets: DiskCleanupTarget[];
    usageInsights: DiskUsageInsight[];
}

export interface DiskCleanupResult {
    releasedBytes: number;
    cleanedTargets: DiskCleanupTarget[];
}

export interface PortUsage {
    protocol: string;
    localAddress: string;
    localPort: number;
    remoteAddress?: string;
    remotePort?: number;
    state: string;
    pid?: number;
    processName: string;
    processPath?: string;
}

export interface PortScan {
    scannedAt: string;
    ports: PortUsage[];
}

export interface NetworkDevice {
    ipAddress: string;
    macAddress?: string;
    hostname?: string;
    interfaceName: string;
    state: string;
    source: string;
    reachable: boolean;
}

export interface NetworkDeviceScan {
    scannedAt: string;
    networkCount: number;
    devices: NetworkDevice[];
}

export interface SshKeyInfo {
    name: string;
    keyType: string;
    publicKeyPath: string;
    privateKeyPath?: string;
    publicKey: string;
    fingerprint?: string;
    comment?: string;
    modifiedAt?: string;
    hasPrivateKey: boolean;
}

export interface SshKeyGenerationRequest {
    fileName: string;
    keyType: string;
    comment: string;
}

export interface DockerAvailability {
    installed: boolean;
    version?: string;
}

export interface DockerContainer {
    id: string;
    name: string;
    image: string;
    parentName?: string;
    serviceName?: string;
    state: string;
    status: string;
    ports: string;
    created: string;
    running: boolean;
}

export interface DockerImage {
    id: string;
    repository: string;
    tag: string;
    size: string;
    created: string;
}

export interface DockerRegistryImage {
    repository: string;
    tags: string[];
}

export interface DockerDashboard {
    installed: boolean;
    running: boolean;
    version?: string;
    serverVersion?: string;
    error?: string;
    containers: DockerContainer[];
    images: DockerImage[];
}

export interface DiskUsageInsightCleanupRequest {
    insightId: string;
}

export interface DiskUsageInsightCleanupResult {
    releasedBytes: number;
    cleanedInsight: DiskUsageInsight;
}

export interface BackendMemoryInfo {
    installedBytes?: number;
    inUseBytes: number;
    compressedBytes?: number;
    availableBytes: number;
    committedBytes: number;
    commitLimitBytes: number;
    cachedBytes: number;
    pagedPoolBytes: number;
    nonPagedPoolBytes: number;
    speedMhz?: number;
    slotsUsed?: number;
    slotsTotal?: number;
    formFactor?: string;
    hardwareReservedBytes?: number;
}

export interface BackendGpuAdapterUsage {
    name: string;
    adapterIndex: number;
    utilizationPercent: number;
    engines: BackendGpuEngineUsage[];
}

export interface BackendGpuEngineUsage {
    name: string;
    utilizationPercent: number;
}

export interface BackendCpuInfo {
    model: string;
    currentSpeedMhz: number;
    baseSpeedMhz: number;
    sockets: number;
    cores: number;
    logicalProcessors: number;
    uptimeSeconds: number;
    totalThreads: number;
    totalHandles?: number;
    virtualization?: string;
    l1CacheBytes?: number;
    l2CacheBytes?: number;
    l3CacheBytes?: number;
}

export interface BackendProcessRow {
    info: {
        pid: number;
        name: string;
        publisher: string;
        status: string;
        user: string;
        path: string;
        hasVisibleWindow: boolean;
        iconDataUrl?: string;
    };
    metrics: {
        cpuPercent: number;
        gpuPercent: number;
        memoryBytes: number;
        diskReadBytes: number;
        diskWrittenBytes: number;
    };
}
