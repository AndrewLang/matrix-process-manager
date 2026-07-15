import { Component, input } from "@angular/core";
import { ProcessRow } from "../../app.models";
import { IconComponent } from "../icon/icon.component";

@Component({
    selector: "mtx-top-resource-consumers",
    imports: [IconComponent],
    templateUrl: "./top-resource-consumers.component.html",
})
export class TopResourceConsumersComponent {
    rows = input.required<ProcessRow[]>();
}