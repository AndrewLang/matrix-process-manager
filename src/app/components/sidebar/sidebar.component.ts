import { NgClass } from "@angular/common";
import { Component, input, output } from "@angular/core";
import { NavItem, ViewId } from "../../app.models";

@Component({
    selector: "app-sidebar",
    imports: [NgClass],
    templateUrl: "./sidebar.component.html",
})
export class SidebarComponent {
    overviewItems = input.required<NavItem[]>();
    toolItems = input.required<NavItem[]>();
    activeView = input.required<ViewId>();
    viewChange = output<ViewId>();
}
