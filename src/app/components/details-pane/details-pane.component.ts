import { Component, input, output } from "@angular/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { ProcessRow, ResourceBar } from "../../app.models";
import { IconComponent } from "../icon/icon.component";
import { ResourceBarsComponent } from "../resource-bars/resource-bars.component";

@Component({
    selector: "mtx-details-pane",
    imports: [IconComponent, ResourceBarsComponent],
    host: { class: "block h-full min-h-0" },
    templateUrl: "./details-pane.component.html",
})
export class DetailsPaneComponent {
    process = input<ProcessRow | undefined>();
    bars = input.required<ResourceBar[]>();
    closeDetails = output<void>();
    endProcess = output<ProcessRow>();

    openProcessLocation(process: ProcessRow): void {
        if (!process.path) {
            return;
        }

        revealItemInDir(process.path).catch(() => undefined);
    }
}