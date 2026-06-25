import { Component, input } from "@angular/core";

@Component({
    selector: "mtx-search-box",
    templateUrl: "./search-box.component.html",
})
export class SearchBoxComponent {
    placeholder = input.required<string>();
    hostClass = input("");
    inputClass = input("h-full flex-1 leading-[30px]");
    shortcut = input<string>();
}
