import { BackendProcessRow, ProcessGroup, ProcessRow } from "./app.models";

export interface ProcessSnapshotWorkerRequest {
    requestId: number;
    processes: BackendProcessRow[];
    selectedPid?: number;
    processOrder: number[];
}

export interface ProcessSnapshotWorkerResponse {
    requestId: number;
    rows: ProcessRow[];
    processOrder: number[];
    diskBytes: number;
}

self.addEventListener("message", (event: MessageEvent<ProcessSnapshotWorkerRequest>) => {
    const orderedProcesses = stabilizeProcessOrder(event.data.processes, event.data.processOrder);
    const rows = orderedProcesses.processes.map((row) => toProcessRow(row, event.data.selectedPid));
    const diskBytes = event.data.processes.reduce((total, row) => total + row.metrics.diskReadBytes + row.metrics.diskWrittenBytes, 0);
    const response: ProcessSnapshotWorkerResponse = {
        requestId: event.data.requestId,
        rows,
        processOrder: orderedProcesses.processOrder,
        diskBytes,
    };
    self.postMessage(response);
});

function toProcessRow(row: BackendProcessRow, selectedPid: number | undefined): ProcessRow {
    return {
        name: row.info.name || `Process ${row.info.pid}`,
        publisher: row.info.publisher || row.info.path || "Unknown publisher",
        processGroup: classifyProcess(row),
        iconDataUrl: row.info.iconDataUrl,
        pid: row.info.pid,
        status: row.info.status,
        cpu: `${row.metrics.cpuPercent.toFixed(1)}%`,
        gpu: `${row.metrics.gpuPercent.toFixed(1)}%`,
        memory: formatBytes(row.metrics.memoryBytes),
        disk: `${formatBytes(row.metrics.diskReadBytes + row.metrics.diskWrittenBytes)}/s`,
        network: "0 Mbps",
        user: row.info.user || "system",
        path: row.info.path,
        iconClass: "bi-window",
        selected: row.info.pid === selectedPid,
    };
}

function stabilizeProcessOrder(processes: BackendProcessRow[], processOrder: number[]): { processes: BackendProcessRow[]; processOrder: number[] } {
    const currentPids = new Set(processes.map((process) => process.info.pid));
    const knownPids = new Set(processOrder);
    const nextProcessOrder = processOrder.filter((pid) => currentPids.has(pid));

    for (const process of processes) {
        if (!knownPids.has(process.info.pid)) {
            nextProcessOrder.push(process.info.pid);
        }
    }

    const order = new Map(nextProcessOrder.map((pid, index) => [pid, index]));
    return {
        processOrder: nextProcessOrder,
        processes: [...processes].sort((left, right) => (order.get(left.info.pid) ?? Number.MAX_SAFE_INTEGER) - (order.get(right.info.pid) ?? Number.MAX_SAFE_INTEGER)),
    };
}

function formatBytes(bytes: number): string {
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

function classifyProcess(row: BackendProcessRow): ProcessGroup {
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