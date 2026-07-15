import { NgClass } from "@angular/common";
import { Component, input, output } from "@angular/core";
import { NavItem, ViewId } from "../../app.models";
import { IconComponent } from "../icon/icon.component";

@Component({
    selector: "mtx-sidebar",
    imports: [NgClass, IconComponent],
    host: { class: "block h-full min-h-0 overflow-hidden" },
    templateUrl: "./sidebar.component.html",
})
export class SidebarComponent {
    overviewItems = input.required<NavItem[]>();
    toolItems = input.required<NavItem[]>();
    activeView = input.required<ViewId>();
    workstationName = input("My Workstation");
    uptime = input("Loading...");
    viewChange = output<ViewId>();
    toolSelected = output<NavItem>();
}
