import { Component, output } from "@angular/core";
import { SearchBoxComponent } from "../search-box/search-box.component";

@Component({
    selector: "mtx-titlebar",
    imports: [SearchBoxComponent],
    templateUrl: "./titlebar.component.html",
})
export class TitlebarComponent {
    dragStart = output<MouseEvent>();
    minimizeWindow = output<void>();
    toggleMaximizeWindow = output<void>();
    closeWindow = output<void>();

    searchClass = "flex h-[24px] translate-y-px items-center gap-2 rounded-[5px] border border-[var(--border)] bg-[rgba(15,28,40,0.84)] py-0 pr-2 pl-[11px] text-[var(--muted)] [width:min(452px,calc(100vw-420px))]";
    searchInputClass = "h-full flex-1 leading-[24px]";
}
