import { Component, OnInit, computed, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { DockerContainer, DockerDashboard, DockerImage } from "../../app.models";
import { DataGridColumn, DataGridComponent } from "../../components/data-grid/data-grid.component";

type DockerTab = "containers" | "images";
type DockerPortLink = { key: string; label: string; url?: string };
type DockerContainerRow =
    | { kind: "group"; key: string; name: string; containers: DockerContainer[]; running: boolean }
    | { kind: "container"; key: string; container: DockerContainer; child: boolean };

@Component({
    selector: "mtx-docker-view",
    imports: [DataGridComponent],
    templateUrl: "./docker-view.component.html",
})
export class DockerViewComponent implements OnInit {
    dashboard = signal<DockerDashboard | undefined>(undefined);
    selectedContainerId = signal("");
    selectedImageId = signal("");
    activeTab = signal<DockerTab>("containers");
    loading = signal(false);
    actionRunning = signal(false);
    error = signal("");
    actionMessage = signal("");
    containerFilter = signal("");
    imageFilter = signal("");
    onlyRunningContainers = signal(false);
    expandedParents = signal<ReadonlySet<string>>(new Set());
    openContainerMenuId = signal("");
    logsContainerId = signal("");
    logsContainerName = signal("");
    containerLogs = signal("");
    logsLoading = signal(false);

    imageColumns: DataGridColumn<DockerImage>[] = [
        { key: "repository", label: "Repository", width: 250, minWidth: 160, align: "left", value: (image) => image.repository, cellClass: () => "font-mono text-[#e6f0fa]" },
        { key: "tag", label: "Tag", width: 140, minWidth: 100, value: (image) => image.tag },
        { key: "id", label: "Image ID", width: 160, minWidth: 120, value: (image) => image.id, cellClass: () => "font-mono text-(--muted)" },
        { key: "size", label: "Size", width: 110, minWidth: 84, value: (image) => image.size },
        { key: "created", label: "Created", width: 130, minWidth: 96, value: (image) => image.created },
    ];
    containerKey = (container: DockerContainer) => container.id;
    imageKey = (image: DockerImage) => image.id;

    containers = computed(() => this.dashboard()?.containers ?? []);
    images = computed(() => this.dashboard()?.images ?? []);
    filteredContainers = computed(() => this.filterContainers());
    containerRows = computed(() => this.buildContainerRows());
    filteredImages = computed(() => this.filterImages());
    runningContainers = computed(() => this.containers().filter((container) => container.running).length);
    stoppedContainers = computed(() => this.containers().filter((container) => !container.running).length);
    selectedContainer = computed(() => this.filteredContainers().find((container) => container.id === this.selectedContainerId()) ?? this.filteredContainers()[0]);
    selectedImage = computed(() => this.filteredImages().find((image) => image.id === this.selectedImageId()) ?? this.filteredImages()[0]);

    ngOnInit(): void {
        this.refresh();
    }

    refresh(): void {
        this.loading.set(true);
        this.error.set("");
        invoke<DockerDashboard>("get_docker_dashboard")
            .then((dashboard) => {
                this.dashboard.set(dashboard);
                this.error.set(dashboard.error ?? "");
                const selectedContainer = dashboard.containers.find((container) => container.id === this.selectedContainerId()) ?? dashboard.containers[0];
                const selectedImage = dashboard.images.find((image) => image.id === this.selectedImageId()) ?? dashboard.images[0];
                this.selectedContainerId.set(selectedContainer?.id ?? "");
                this.selectedImageId.set(selectedImage?.id ?? "");
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Docker dashboard could not be loaded."))
            .finally(() => this.loading.set(false));
    }

    setActiveTab(tab: DockerTab): void {
        this.activeTab.set(tab);
    }

    setContainerFilter(value: string): void {
        this.containerFilter.set(value);
    }

    toggleOnlyRunningContainers(): void {
        this.onlyRunningContainers.update((value) => !value);
    }

    toggleParent(parentKey: string): void {
        this.expandedParents.update((parents) => {
            const next = new Set(parents);
            if (next.has(parentKey)) {
                next.delete(parentKey);
            } else {
                next.add(parentKey);
            }

            return next;
        });
    }

    parentExpanded(parentKey: string): boolean {
        return this.expandedParents().has(parentKey);
    }

    setImageFilter(value: string): void {
        this.imageFilter.set(value);
    }

    selectContainer(container: DockerContainer): void {
        this.selectedContainerId.set(container.id);
        this.actionMessage.set("");
    }

    toggleContainerMenu(event: MouseEvent, container: DockerContainer): void {
        event.stopPropagation();
        this.openContainerMenuId.update((id) => id === container.id ? "" : container.id);
        this.selectContainer(container);
    }

    closeContainerMenu(): void {
        this.openContainerMenuId.set("");
    }

    selectImage(image: DockerImage): void {
        this.selectedImageId.set(image.id);
    }

    runContainerAction(action: "start" | "stop" | "restart", container: DockerContainer | undefined): void {
        if (!container || this.actionRunning()) {
            return;
        }

        this.actionRunning.set(true);
        this.error.set("");
        this.actionMessage.set("");
        invoke<void>("run_docker_container_action", { containerId: container.id, action })
            .then(() => {
                this.actionMessage.set(`${container.name} ${action} requested.`);
                this.refresh();
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : `Docker ${action} failed.`))
            .finally(() => this.actionRunning.set(false));
    }

    runContainerGroupAction(action: "start" | "stop" | "restart", row: DockerContainerRow): void {
        if (row.kind !== "group" || this.actionRunning()) {
            return;
        }

        const targets = row.containers.filter((container) => action === "start" ? !container.running : container.running);
        if (targets.length === 0) {
            return;
        }

        this.actionRunning.set(true);
        this.error.set("");
        this.actionMessage.set("");
        Promise.all(targets.map((container) => invoke<void>("run_docker_container_action", { containerId: container.id, action })))
            .then(() => {
                this.actionMessage.set(`${row.name} ${action} requested.`);
                this.refresh();
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : `Docker ${action} failed.`))
            .finally(() => this.actionRunning.set(false));
    }

    removeContainer(container: DockerContainer | undefined): void {
        if (!container || this.actionRunning()) {
            return;
        }

        this.closeContainerMenu();
        this.actionRunning.set(true);
        this.error.set("");
        this.actionMessage.set("");
        invoke<void>("run_docker_container_action", { containerId: container.id, action: "forceRemove" })
            .then(() => {
                this.actionMessage.set(`${container.name} removed.`);
                this.refresh();
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Docker remove failed."))
            .finally(() => this.actionRunning.set(false));
    }

    removeContainerGroup(row: DockerContainerRow): void {
        if (row.kind !== "group" || this.actionRunning()) {
            return;
        }

        this.closeContainerMenu();
        this.actionRunning.set(true);
        this.error.set("");
        this.actionMessage.set("");
        Promise.all(row.containers.map((container) => invoke<void>("run_docker_container_action", { containerId: container.id, action: "forceRemove" })))
            .then(() => {
                this.actionMessage.set(`${row.name} removed.`);
                this.refresh();
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Docker remove failed."))
            .finally(() => this.actionRunning.set(false));
    }

    removeImage(image: DockerImage | undefined): void {
        if (!image || this.actionRunning()) {
            return;
        }

        this.actionRunning.set(true);
        this.error.set("");
        this.actionMessage.set("");
        invoke<void>("remove_docker_image", { imageId: image.id })
            .then(() => {
                this.actionMessage.set(`${image.repository}:${image.tag} removed.`);
                this.refresh();
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Docker image remove failed."))
            .finally(() => this.actionRunning.set(false));
    }

    copyContainerId(container: DockerContainer | undefined): void {
        if (!container) {
            return;
        }

        this.closeContainerMenu();
        navigator.clipboard.writeText(container.id).then(
            () => this.actionMessage.set("Container ID copied."),
            () => this.actionMessage.set("Container ID could not be copied."),
        );
    }

    hasContainerMenuItems(container: DockerContainer | undefined): boolean {
        return Boolean(container);
    }

    copyContainerImage(container: DockerContainer | undefined): void {
        if (!container) {
            return;
        }

        this.closeContainerMenu();
        navigator.clipboard.writeText(container.image).then(
            () => this.actionMessage.set("Image name copied."),
            () => this.actionMessage.set("Image name could not be copied."),
        );
    }

    copyContainerInspect(container: DockerContainer | undefined): void {
        if (!container || this.actionRunning()) {
            return;
        }

        this.closeContainerMenu();
        this.actionRunning.set(true);
        invoke<string>("get_docker_container_inspect", { containerId: container.id })
            .then((inspect) => navigator.clipboard.writeText(inspect))
            .then(() => this.actionMessage.set("Inspect JSON copied."))
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Docker inspect failed."))
            .finally(() => this.actionRunning.set(false));
    }

    copyContainerLogs(container: DockerContainer | undefined): void {
        if (!container || this.actionRunning()) {
            return;
        }

        this.closeContainerMenu();
        this.actionRunning.set(true);
        invoke<string>("get_docker_container_logs", { containerId: container.id })
            .then((logs) => navigator.clipboard.writeText(logs))
            .then(() => this.actionMessage.set("Recent logs copied."))
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Docker logs failed."))
            .finally(() => this.actionRunning.set(false));
    }

    viewContainerLogs(event: MouseEvent, container: DockerContainer): void {
        event.stopPropagation();
        if (this.logsLoading()) {
            return;
        }

        this.selectContainer(container);
        this.logsContainerId.set(container.id);
        this.logsContainerName.set(container.name);
        this.containerLogs.set("");
        this.logsLoading.set(true);
        this.error.set("");
        invoke<string>("get_docker_container_logs", { containerId: container.id })
            .then((logs) => this.containerLogs.set(logs.trim() || "No logs returned."))
            .catch((error: unknown) => {
                this.containerLogs.set("");
                this.error.set(error instanceof Error ? error.message : "Docker logs failed.");
            })
            .finally(() => this.logsLoading.set(false));
    }

    closeContainerLogs(): void {
        this.logsContainerId.set("");
        this.logsContainerName.set("");
        this.containerLogs.set("");
    }

    containerRowSelected(row: DockerContainerRow): boolean {
        return row.kind === "container" && row.container.id === this.selectedContainerId();
    }

    containerRowRunning(row: DockerContainerRow): boolean {
        return row.kind === "group" ? row.running : row.container.running;
    }

    containerRowStatusLabel(row: DockerContainerRow): string {
        if (row.kind === "group") {
            const running = row.containers.filter((container) => container.running).length;
            return `${running}/${row.containers.length} running`;
        }

        return row.container.state || "Unknown";
    }

    containerRowName(row: DockerContainerRow): string {
        return row.kind === "group" ? row.name : row.container.name;
    }

    containerRowId(row: DockerContainerRow): string {
        return row.kind === "group" ? "-" : row.container.id;
    }

    containerRowImage(row: DockerContainerRow): string {
        return row.kind === "group" ? "-" : row.container.image;
    }

    containerRowPorts(row: DockerContainerRow): string {
        return row.kind === "group" ? "-" : row.container.ports || "-";
    }

    containerRowPortLinks(row: DockerContainerRow): DockerPortLink[] {
        return row.kind === "group" ? [{ key: row.key, label: "-" }] : this.portLinks(row.container.ports);
    }

    containerPortLinks(container: DockerContainer): DockerPortLink[] {
        return this.portLinks(container.ports);
    }

    openPort(event: MouseEvent, url: string | undefined): void {
        event.stopPropagation();
        if (!url) {
            return;
        }

        openUrl(url).catch((error: unknown) => {
            this.error.set(error instanceof Error ? error.message : "Port link could not be opened.");
        });
    }

    private filterContainers(): DockerContainer[] {
        const query = this.containerFilter().trim().toLowerCase();
        return this.containers().filter((container) => (!this.onlyRunningContainers() || container.running) && (!query || [
            container.parentName ?? "Standalone",
            container.name,
            container.serviceName ?? "",
            container.image,
            container.state,
            container.status,
            container.ports,
            container.id,
        ].some((value) => value.toLowerCase().includes(query))));
    }

    private buildContainerRows(): DockerContainerRow[] {
        const groups = new Map<string, DockerContainer[]>();
        const standalone: DockerContainer[] = [];

        for (const container of this.filteredContainers()) {
            if (container.parentName) {
                const containers = groups.get(container.parentName) ?? [];
                containers.push(container);
                groups.set(container.parentName, containers);
            } else {
                standalone.push(container);
            }
        }

        const rows: DockerContainerRow[] = [];
        for (const [parentName, containers] of [...groups.entries()].sort(([left], [right]) => left.localeCompare(right))) {
            const key = `parent:${parentName}`;
            rows.push({ kind: "group", key, name: parentName, containers, running: containers.some((container) => container.running) });
            if (this.parentExpanded(key)) {
                rows.push(...containers.map((container) => ({ kind: "container" as const, key: container.id, container, child: true })));
            }
        }

        rows.push(...standalone.map((container) => ({ kind: "container" as const, key: container.id, container, child: false })));
        return rows;
    }

    private filterImages(): DockerImage[] {
        const query = this.imageFilter().trim().toLowerCase();
        if (!query) {
            return this.images();
        }

        return this.images().filter((image) => [
            image.repository,
            image.tag,
            image.id,
            image.size,
            image.created,
        ].some((value) => value.toLowerCase().includes(query)));
    }

    private portLinks(ports: string): DockerPortLink[] {
        if (!ports.trim()) {
            return [{ key: "none", label: "-" }];
        }

        const links = new Map<string, DockerPortLink>();
        for (const segment of ports.split(",")) {
            const port = segment.trim();
            if (!port) {
                continue;
            }

            const [hostPart, containerPart] = port.split("->").map((part) => part.trim());
            const hostPort = hostPart?.match(/(\d+)$/)?.[1];
            const containerPort = containerPart?.match(/^(\d+)/)?.[1];
            const key = hostPort && containerPart ? `${hostPort}->${containerPart}` : port;
            const protocol = hostPort === "443" || containerPort === "443" ? "https" : "http";

            links.set(key, {
                key,
                label: hostPort && containerPart ? `${hostPort}:${containerPart}` : port,
                url: hostPort ? `${protocol}://localhost:${hostPort}` : undefined,
            });
        }

        return links.size > 0 ? [...links.values()] : [{ key: "none", label: "-" }];
    }
}
