import { Component, input, output } from "@angular/core";

@Component({
    selector: "mtx-common-dialog",
    templateUrl: "./common-dialog.component.html",
})
export class CommonDialogComponent {
    title = input.required<string>();
    panelClass = input("w-[min(420px,100%)]");
    contentClass = input("p-3.5");
    closeDialog = output<void>();
}