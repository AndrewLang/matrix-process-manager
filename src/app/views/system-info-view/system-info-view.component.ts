import { Component, inject } from "@angular/core";
import { WorkareaStateService } from "../../services/workarea-state.service";

@Component({
    selector: "mtx-system-info-view",
    templateUrl: "./system-info-view.component.html",
})
export class SystemInfoViewComponent {
    state = inject(WorkareaStateService);
}