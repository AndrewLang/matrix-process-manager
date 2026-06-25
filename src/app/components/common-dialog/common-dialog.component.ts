import { Component, input, output } from "@angular/core";

@Component({
    selector: "mtx-common-dialog",
    templateUrl: "./common-dialog.component.html",
})
export class CommonDialogComponent {
    title = input.required<string>();
    closeDialog = output<void>();
}