import { NgClass } from "@angular/common";
import { Component, inject } from "@angular/core";
import { UpdateFrequency } from "../../app.models";
import { WorkareaStateService } from "../../services/workarea-state.service";

@Component({
    selector: "mtx-settings-view",
    imports: [NgClass],
    templateUrl: "./settings-view.component.html",
})
export class SettingsViewComponent {
    state = inject(WorkareaStateService);
    categories = [
        { label: "General", icon: "bi-gear" },
        { label: "Appearance", icon: "bi-brush" },
        { label: "Process Table", icon: "bi-table" },
        { label: "Notifications", icon: "bi-bell" },
        { label: "Performance", icon: "bi-graph-up" },
        { label: "Startup & Update", icon: "bi-cloud-arrow-up" },
        { label: "Data & Storage", icon: "bi-database" },
        { label: "Integrations", icon: "bi-puzzle" },
        { label: "Advanced", icon: "bi-code-slash" },
        { label: "About", icon: "bi-info-circle" },
    ];
    frequencies: { value: UpdateFrequency; label: string; detail: string }[] = [
        { value: "high", label: "High", detail: "Refresh every second" },
        { value: "normal", label: "Normal", detail: "Refresh every 2 seconds" },
        { value: "low", label: "Low", detail: "Refresh every 5 seconds" },
        { value: "paused", label: "Paused", detail: "Stop automatic updates" },
    ];
}