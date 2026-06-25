export type ViewId = "dashboard" | "processes" | "performance" | "startup" | "system" | "settings" | "disk" | "terminal" | "more";
export type ProcessGroup = "apps" | "background" | "windows";
export type UpdateFrequency = "high" | "normal" | "low" | "paused";

export interface NavItem {
    id: ViewId;
    label: string;
    icon: string;
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
    status: string;
    impact: string;
    startupType: string;
    source: string;
    command: string;
    path: string;
    delaySeconds?: number;
}

export interface BackendProcessSnapshot {
    processes: BackendProcessRow[];
    totalProcesses: number;
}

export interface BackendProcessRow {
    info: {
        pid: number;
        name: string;
        publisher: string;
        status: string;
        user: string;
        path: string;
        iconDataUrl?: string;
    };
    metrics: {
        cpuPercent: number;
        memoryBytes: number;
        diskReadBytes: number;
        diskWrittenBytes: number;
    };
}
