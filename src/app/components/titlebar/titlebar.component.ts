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

    searchClass = "flex h-8 translate-y-px items-center gap-2.5 rounded-md border border-[rgba(122,153,181,0.24)] bg-[rgba(28,43,56,0.72)] py-0 pr-2.5 pl-3 text-[12px] text-[#aebdca] shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] hover:bg-[rgba(35,52,67,0.78)] [width:min(460px,calc(100vw-430px))]";
    searchInputClass = "h-full flex-1 leading-8 placeholder:text-[#8292a1]";
}
