import { Routes } from "@angular/router";
import { DashboardViewComponent } from "./views/dashboard-view/dashboard-view.component";
import { PerformanceViewComponent } from "./views/performance-view/performance-view.component";
import { PlaceholderViewComponent } from "./views/placeholder-view/placeholder-view.component";
import { ProcessesViewComponent } from "./views/processes-view/processes-view.component";

export const routes: Routes = [
    { path: "", pathMatch: "full", redirectTo: "dashboard" },
    { path: "dashboard", component: DashboardViewComponent },
    { path: "processes", component: ProcessesViewComponent },
    { path: "performance", component: PerformanceViewComponent },
    { path: "startup", component: PlaceholderViewComponent, data: { title: "Startup Apps" } },
    { path: "users", component: PlaceholderViewComponent, data: { title: "Users" } },
    { path: "services", component: PlaceholderViewComponent, data: { title: "Services" } },
    { path: "system", component: PlaceholderViewComponent, data: { title: "System Info" } },
    { path: "logs", component: PlaceholderViewComponent, data: { title: "Logs" } },
    { path: "settings", component: PlaceholderViewComponent, data: { title: "Settings" } },
];
