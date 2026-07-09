import { Routes } from "@angular/router";
import { CommandCenterViewComponent } from "./views/command-center/command-center-view.component";
import { DashboardViewComponent } from "./views/dashboard-view/dashboard-view.component";
import { DiskViewComponent } from "./views/disk-view/disk-view.component";
import { DockerViewComponent } from "./views/docker-view/docker-view.component";
import { NetworkDevicesViewComponent } from "./views/network-devices-view/network-devices-view.component";
import { CpuMonitorComponent } from "./views/performance-view/components/cpu-monitor/cpu-monitor.component";
import { DiskMonitorComponent } from "./views/performance-view/components/disk-monitor/disk-monitor.component";
import { GpuMonitorComponent } from "./views/performance-view/components/gpu-monitor/gpu-monitor.component";
import { MemoryMonitorComponent } from "./views/performance-view/components/memory-monitor/memory-monitor.component";
import { NetworkMonitorComponent } from "./views/performance-view/components/network-monitor/network-monitor.component";
import { PerformanceViewComponent } from "./views/performance-view/performance-view.component";
import { PlaceholderViewComponent } from "./views/placeholder-view/placeholder-view.component";
import { PortsViewComponent } from "./views/ports-view/ports-view.component";
import { ProcessesViewComponent } from "./views/processes-view/processes-view.component";
import { SettingsViewComponent } from "./views/settings-view/settings-view.component";
import { SshKeysViewComponent } from "./views/ssh-keys-view/ssh-keys-view.component";
import { StartupViewComponent } from "./views/startup-view/startup-view.component";
import { SystemInfoViewComponent } from "./views/system-info-view/system-info-view.component";

export const routes: Routes = [
    { path: "", pathMatch: "full", redirectTo: "dashboard" },
    { path: "dashboard", component: DashboardViewComponent },
    { path: "processes", component: ProcessesViewComponent },
    {
        path: "performance",
        component: PerformanceViewComponent,
        children: [
            { path: "", pathMatch: "full", redirectTo: "cpu" },
            { path: "cpu", component: CpuMonitorComponent },
            { path: "gpu", component: GpuMonitorComponent },
            { path: "memory", component: MemoryMonitorComponent },
            { path: "network", component: NetworkMonitorComponent },
            { path: "disk", component: DiskMonitorComponent },
        ],
    },
    { path: "startup", component: StartupViewComponent },
    { path: "system", component: SystemInfoViewComponent },
    { path: "command-center", component: CommandCenterViewComponent },
    { path: "settings", component: SettingsViewComponent },
    { path: "storage", component: DiskViewComponent },
    { path: "ports", component: PortsViewComponent },
    { path: "network-devices", component: NetworkDevicesViewComponent },
    { path: "ssh-keys", component: SshKeysViewComponent },
    { path: "docker", component: DockerViewComponent },
    { path: "disk", pathMatch: "full", redirectTo: "storage" },
    { path: "terminal", component: PlaceholderViewComponent, data: { title: "Terminal" } },
    { path: "more", component: PlaceholderViewComponent, data: { title: "More Tools" } },
];
