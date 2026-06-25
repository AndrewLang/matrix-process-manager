import { NgClass } from "@angular/common";
import { Component, computed, inject, signal } from "@angular/core";
import { toSignal } from "@angular/core/rxjs-interop";
import { NavigationEnd, Router, RouterOutlet } from "@angular/router";
import { filter, map } from "rxjs";
import { MiniConsumersComponent } from "../../components/mini-consumers/mini-consumers.component";
import { ResourceBarsComponent } from "../../components/resource-bars/resource-bars.component";
import { WorkareaStateService } from "../../services/workarea-state.service";
import { PerformanceResourceNavComponent } from "./components/performance-resource-nav/performance-resource-nav.component";
import { PerformanceMetric, PerformanceNavItem } from "./performance-view.models";

@Component({
    selector: "mtx-performance-view",
    imports: [NgClass, RouterOutlet, MiniConsumersComponent, ResourceBarsComponent, PerformanceResourceNavComponent],
    templateUrl: "./performance-view.component.html",
})
export class PerformanceViewComponent {
    state = inject(WorkareaStateService);
    private router = inject(Router);
    detailsOpen = signal(true);
    private currentUrl = toSignal(this.router.events.pipe(filter((event): event is NavigationEnd => event instanceof NavigationEnd), map((event) => event.urlAfterRedirects)), { initialValue: this.router.url });
    selectedMetric = computed<PerformanceMetric>(() => this.metricFromUrl(this.currentUrl()));

    resourceNav: PerformanceNavItem[] = [
        { key: "cpu", label: "CPU", icon: "bi-cpu", accent: "text-(--blue)" },
        { key: "gpu", label: "GPU", icon: "bi-gpu-card", accent: "text-(--cyan)" },
        { key: "memory", label: "Memory", icon: "bi-memory", accent: "text-(--violet)" },
        { key: "network", label: "Network", icon: "bi-ethernet", accent: "text-(--orange)" },
        { key: "disk", label: "Disk", icon: "bi-device-hdd", accent: "text-(--green)" },
    ];

    setSelectedMetric(metric: PerformanceMetric): void {
        this.router.navigate(["performance", metric]);
    }

    private metricFromUrl(url: string): PerformanceMetric {
        const metric = url.split("?")[0].split("/").filter(Boolean)[1];
        return this.isPerformanceMetric(metric) ? metric : "cpu";
    }

    private isPerformanceMetric(value: string | undefined): value is PerformanceMetric {
        return value === "cpu" || value === "gpu" || value === "memory" || value === "network" || value === "disk";
    }

    closeDetails(): void {
        this.detailsOpen.set(false);
    }
}