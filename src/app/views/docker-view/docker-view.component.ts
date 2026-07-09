import { Component, OnInit, computed, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { DockerContainer, DockerDashboard, DockerImage, DockerRegistryImage } from "../../app.models";
import { CopyButtonComponent } from "../../components/copy-button/copy-button.component";
import { DataGridColumn, DataGridComponent } from "../../components/data-grid/data-grid.component";

type DockerTab = "containers" | "images" | "registry" | "remote";
type DockerPortLink = { key: string; label: string; url?: string };
type DockerContainerRow =
    | { kind: "group"; key: string; name: string; containers: DockerContainer[]; running: boolean }
    | { kind: "container"; key: string; container: DockerContainer; child: boolean };
type DockerRegistryProfile = { registry: string; username: string; password: string };

@Component({
    selector: "mtx-docker-view",
    imports: [CopyButtonComponent, DataGridComponent],
    templateUrl: "./docker-view.component.html",
})
export class DockerViewComponent implements OnInit {
    private readonly savedRegistriesKey = "matrixProcessManager.docker.savedRegistries";
    private readonly dockerHostKey = "matrixProcessManager.docker.host";
    private readonly savedDockerHostsKey = "matrixProcessManager.docker.savedHosts";
    private readonly onlyRunningContainersKey = "matrixProcessManager.docker.onlyRunningContainers";

    dashboard = signal<DockerDashboard | undefined>(undefined);
    private dashboardDockerHost = signal("");
    private refreshRun = 0;
    dockerHost = signal("");
    savedDockerHosts = signal<string[]>([]);
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
    registryUrl = signal("");
    registryUsername = signal("");
    registryPassword = signal("");
    registryFilter = signal("");
    registryImages = signal<DockerRegistryImage[]>([]);
    registryLoading = signal(false);
    selectedRegistryRepository = signal("");
    savedRegistries = signal<DockerRegistryProfile[]>([]);

    imageColumns: DataGridColumn<DockerImage>[] = [
        { key: "repository", label: "Repository", width: 250, minWidth: 160, align: "left", value: (image) => image.repository, cellClass: () => "font-mono text-[#e6f0fa]" },
        { key: "tag", label: "Tag", width: 140, minWidth: 100, value: (image) => image.tag },
        { key: "id", label: "Image ID", width: 160, minWidth: 120, value: (image) => image.id, cellClass: () => "font-mono text-(--muted)" },
        { key: "size", label: "Size", width: 110, minWidth: 84, value: (image) => image.size },
        { key: "created", label: "Created", width: 130, minWidth: 96, value: (image) => image.created },
    ];
    registryColumns: DataGridColumn<DockerRegistryImage>[] = [
        { key: "repository", label: "Repository", width: 320, minWidth: 180, align: "left", value: (image) => image.repository, cellClass: () => "font-mono text-[#e6f0fa]" },
        { key: "tagCount", label: "Tags", width: 90, minWidth: 70, value: (image) => image.tags.length },
        { key: "tagList", label: "Tag List", width: 420, minWidth: 220, value: (image) => this.registryTagsText(image), cellClass: () => "font-mono text-(--muted)" },
    ];
    containerKey = (container: DockerContainer) => container.id;
    imageKey = (image: DockerImage) => image.id;
    registryImageKey = (image: DockerRegistryImage) => image.repository;

    containers = computed(() => this.dashboard()?.containers ?? []);
    images = computed(() => this.dashboard()?.images ?? []);
    filteredContainers = computed(() => this.filterContainers());
    containerRows = computed(() => this.buildContainerRows());
    filteredImages = computed(() => this.filterImages());
    filteredRegistryImages = computed(() => this.filterRegistryImages());
    runningContainers = computed(() => this.containers().filter((container) => container.running).length);
    stoppedContainers = computed(() => this.containers().filter((container) => !container.running).length);
    selectedContainer = computed(() => this.filteredContainers().find((container) => container.id === this.selectedContainerId()) ?? this.filteredContainers()[0]);
    selectedImage = computed(() => this.filteredImages().find((image) => image.id === this.selectedImageId()) ?? this.filteredImages()[0]);
    selectedRegistryImage = computed(() => this.filteredRegistryImages().find((image) => image.repository === this.selectedRegistryRepository()) ?? this.filteredRegistryImages()[0]);

    ngOnInit(): void {
        this.loadSavedDockerHosts();
        this.loadDockerHost();
        this.loadOnlyRunningContainers();
        this.loadSavedRegistries();
        this.refresh();
    }

    refresh(): void {
        const dockerHost = this.currentDockerHost();
        const refreshRun = ++this.refreshRun;
        if (this.dashboardDockerHost() !== dockerHost) {
            this.dashboard.set(undefined);
            this.selectedContainerId.set("");
            this.selectedImageId.set("");
        }
        this.loading.set(true);
        this.error.set("");
        invoke<DockerDashboard>("get_docker_dashboard", { dockerHost })
            .then((dashboard) => {
                if (refreshRun !== this.refreshRun) {
                    return;
                }

                this.dashboard.set(dashboard);
                this.dashboardDockerHost.set(dockerHost);
                this.error.set(dashboard.error ?? "");
                const selectedContainer = dashboard.containers.find((container) => container.id === this.selectedContainerId()) ?? dashboard.containers[0];
                const selectedImage = dashboard.images.find((image) => image.id === this.selectedImageId()) ?? dashboard.images[0];
                this.selectedContainerId.set(selectedContainer?.id ?? "");
                this.selectedImageId.set(selectedImage?.id ?? "");
            })
            .catch((error: unknown) => {
                if (refreshRun === this.refreshRun) {
                    this.error.set(error instanceof Error ? error.message : "Docker dashboard could not be loaded.");
                }
            })
            .finally(() => {
                if (refreshRun === this.refreshRun) {
                    this.loading.set(false);
                }
            });
    }

    setActiveTab(tab: DockerTab): void {
        this.activeTab.set(tab);
        if ((tab === "containers" || tab === "images" || tab === "remote") && this.dashboardDockerHost() !== this.currentDockerHost()) {
            this.refresh();
        }
    }

    setDockerHost(value: string): void {
        this.dockerHost.set(value);
    }

    saveDockerHost(): void {
        const target = this.dockerHost().trim();
        if (!target) {
            return;
        }

        localStorage.setItem(this.dockerHostKey, target);
        const hosts = [target, ...this.savedDockerHosts().filter((host) => host !== target)]
            .sort((left, right) => left.localeCompare(right));
        this.savedDockerHosts.set(hosts);
        this.persistSavedDockerHosts();
        this.actionMessage.set("Remote Docker target connected.");
        this.refresh();
    }

    removeSavedDockerHost(): void {
        const target = this.dockerHost().trim();
        if (!target) {
            return;
        }

        this.savedDockerHosts.set(this.savedDockerHosts().filter((host) => host !== target));
        this.persistSavedDockerHosts();
        if (this.dockerHost().trim() === target) {
            this.dockerHost.set("");
            localStorage.setItem(this.dockerHostKey, "");
        }

        this.actionMessage.set("Remote Docker target removed.");
        this.refresh();
    }

    setContainerFilter(value: string): void {
        this.containerFilter.set(value);
    }

    toggleOnlyRunningContainers(): void {
        const enabled = !this.onlyRunningContainers();
        this.onlyRunningContainers.set(enabled);
        localStorage.setItem(this.onlyRunningContainersKey, String(enabled));
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

    setRegistryUrl(value: string): void {
        this.registryUrl.set(value);
        const profile = this.savedRegistries().find((item) => item.registry === value.trim());
        if (profile) {
            this.registryUsername.set(profile.username);
            this.registryPassword.set(profile.password);
        }
    }

    setRegistryUsername(value: string): void {
        this.registryUsername.set(value);
    }

    setRegistryPassword(value: string): void {
        this.registryPassword.set(value);
    }

    setRegistryFilter(value: string): void {
        this.registryFilter.set(value);
    }

    saveRegistry(): boolean {
        const registry = this.registryUrl().trim();
        if (!registry) {
            return false;
        }

        const profile: DockerRegistryProfile = {
            registry,
            username: this.registryUsername().trim(),
            password: this.registryPassword(),
        };
        const profiles = [profile, ...this.savedRegistries().filter((item) => item.registry !== registry)]
            .sort((left, right) => left.registry.localeCompare(right.registry));
        this.savedRegistries.set(profiles);
        this.persistSavedRegistries();
        return true;
    }

    removeSavedRegistry(): void {
        const registry = this.registryUrl().trim();
        if (!registry) {
            return;
        }

        const profiles = this.savedRegistries().filter((item) => item.registry !== registry);
        this.savedRegistries.set(profiles);
        this.persistSavedRegistries();
        this.registryUrl.set("");
        this.registryUsername.set("");
        this.registryPassword.set("");
        this.actionMessage.set("Registry removed.");
    }

    registrySaved(): boolean {
        const registry = this.registryUrl().trim();
        return this.savedRegistries().some((profile) => profile.registry === registry);
    }

    selectRegistryImage(image: DockerRegistryImage): void {
        this.selectedRegistryRepository.set(image.repository);
    }

    listRegistryImages(): void {
        const registry = this.registryUrl().trim();
        if (!registry || this.registryLoading()) {
            return;
        }

        this.saveRegistry();
        this.registryLoading.set(true);
        this.error.set("");
        this.actionMessage.set("");
        invoke<DockerRegistryImage[]>("list_docker_registry_images", {
            request: {
                registry,
                username: this.registryUsername(),
                password: this.registryPassword(),
            },
        })
            .then((images) => {
                this.registryImages.set(images);
                this.selectedRegistryRepository.set(images[0]?.repository ?? "");
                this.actionMessage.set(`${images.length} registry images loaded. Registry saved.`);
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Docker registry images could not be loaded."))
            .finally(() => this.registryLoading.set(false));
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
        invoke<void>("run_docker_container_action", { containerId: container.id, action, dockerHost: this.currentDockerHost() })
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
        Promise.all(targets.map((container) => invoke<void>("run_docker_container_action", { containerId: container.id, action, dockerHost: this.currentDockerHost() })))
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
        invoke<void>("run_docker_container_action", { containerId: container.id, action: "forceRemove", dockerHost: this.currentDockerHost() })
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
        Promise.all(row.containers.map((container) => invoke<void>("run_docker_container_action", { containerId: container.id, action: "forceRemove", dockerHost: this.currentDockerHost() })))
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
        invoke<void>("remove_docker_image", { imageId: image.id, dockerHost: this.currentDockerHost() })
            .then(() => {
                this.actionMessage.set(`${image.repository}:${image.tag} removed.`);
                this.refresh();
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Docker image remove failed."))
            .finally(() => this.actionRunning.set(false));
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
        invoke<string>("get_docker_container_inspect", { containerId: container.id, dockerHost: this.currentDockerHost() })
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
        invoke<string>("get_docker_container_logs", { containerId: container.id, dockerHost: this.currentDockerHost() })
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
        invoke<string>("get_docker_container_logs", { containerId: container.id, dockerHost: this.currentDockerHost() })
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

    private filterRegistryImages(): DockerRegistryImage[] {
        const query = this.registryFilter().trim().toLowerCase();
        if (!query) {
            return this.registryImages();
        }

        return this.registryImages().filter((image) => [
            image.repository,
            this.registryTagsText(image),
        ].some((value) => value.toLowerCase().includes(query)));
    }

    registryTagsText(image: DockerRegistryImage | undefined): string {
        return image?.tags.length ? image.tags.join(", ") : "-";
    }

    private loadSavedRegistries(): void {
        try {
            const profiles = JSON.parse(localStorage.getItem(this.savedRegistriesKey) ?? "[]") as DockerRegistryProfile[];
            const validProfiles = profiles.filter((profile) => typeof profile.registry === "string" && profile.registry.trim());
            this.savedRegistries.set(validProfiles);
            const firstProfile = validProfiles[0];
            if (firstProfile) {
                this.registryUrl.set(firstProfile.registry);
                this.registryUsername.set(firstProfile.username ?? "");
                this.registryPassword.set(firstProfile.password ?? "");
            }
        } catch {
            this.savedRegistries.set([]);
        }
    }

    private persistSavedRegistries(): void {
        localStorage.setItem(this.savedRegistriesKey, JSON.stringify(this.savedRegistries()));
    }

    private loadSavedDockerHosts(): void {
        try {
            const hosts = JSON.parse(localStorage.getItem(this.savedDockerHostsKey) ?? "[]") as string[];
            this.savedDockerHosts.set(hosts.filter((host) => typeof host === "string" && host.trim()).map((host) => host.trim()));
        } catch {
            this.savedDockerHosts.set([]);
        }
    }

    private persistSavedDockerHosts(): void {
        localStorage.setItem(this.savedDockerHostsKey, JSON.stringify(this.savedDockerHosts()));
    }

    private loadDockerHost(): void {
        const target = localStorage.getItem(this.dockerHostKey) ?? "";
        this.dockerHost.set(target);
    }

    private loadOnlyRunningContainers(): void {
        this.onlyRunningContainers.set(localStorage.getItem(this.onlyRunningContainersKey) === "true");
    }

    private currentDockerHost(): string {
        return this.activeTab() === "remote" ? this.dockerHost().trim() : "";
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
                url: hostPort ? `${protocol}://${this.dockerUrlHost()}:${hostPort}` : undefined,
            });
        }

        return links.size > 0 ? [...links.values()] : [{ key: "none", label: "-" }];
    }

    private dockerUrlHost(): string {
        const target = this.currentDockerHost();
        if (!target) {
            return "localhost";
        }

        return target.split("@").pop()?.split(":")[0] || "localhost";
    }
}
