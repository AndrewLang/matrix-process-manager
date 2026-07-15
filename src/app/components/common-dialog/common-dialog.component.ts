import { Component, input, output } from "@angular/core";
import { IconComponent } from "../icon/icon.component";

@Component({
    selector: "mtx-common-dialog",
    imports: [IconComponent],
    templateUrl: "./common-dialog.component.html",
})
export class CommonDialogComponent {
    title = input.required<string>();
    panelClass = input("w-[min(420px,100%)]");
    contentClass = input("p-3.5");
    closeDialog = output<void>();
}