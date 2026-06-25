export type ViewId = "dashboard" | "processes" | "performance" | "startup" | "users" | "services" | "system" | "logs" | "settings";
export type ProcessGroup = "apps" | "background" | "windows";

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
