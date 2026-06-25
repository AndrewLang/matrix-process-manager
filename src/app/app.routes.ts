import { Routes } from "@angular/router";
import { DashboardViewComponent } from "./views/dashboard-view/dashboard-view.component";
import { PerformanceViewComponent } from "./views/performance-view/performance-view.component";
import { PlaceholderViewComponent } from "./views/placeholder-view/placeholder-view.component";
import { ProcessesViewComponent } from "./views/processes-view/processes-view.component";
import { SettingsViewComponent } from "./views/settings-view/settings-view.component";
import { StartupViewComponent } from "./views/startup-view/startup-view.component";
import { SystemInfoViewComponent } from "./views/system-info-view/system-info-view.component";

export const routes: Routes = [
    { path: "", pathMatch: "full", redirectTo: "dashboard" },
    { path: "dashboard", component: DashboardViewComponent },
    { path: "processes", component: ProcessesViewComponent },
    { path: "performance", component: PerformanceViewComponent },
    { path: "startup", component: StartupViewComponent },
    { path: "system", component: SystemInfoViewComponent },
    { path: "settings", component: SettingsViewComponent },
    { path: "disk", component: PlaceholderViewComponent, data: { title: "Disk Manager" } },
    { path: "terminal", component: PlaceholderViewComponent, data: { title: "Terminal" } },
    { path: "more", component: PlaceholderViewComponent, data: { title: "More Tools" } },
];
