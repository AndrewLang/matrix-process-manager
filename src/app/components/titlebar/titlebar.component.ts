import { NgClass } from "@angular/common";
import { Component, output } from "@angular/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Action } from "../../app.models";
import { IconComponent } from "../icon/icon.component";
import { SearchBoxComponent } from "../search-box/search-box.component";

@Component({
    selector: "mtx-titlebar",
    imports: [SearchBoxComponent, IconComponent, NgClass],
    templateUrl: "./titlebar.component.html",
})
export class TitlebarComponent {
    dragStart = output<MouseEvent>();
    minimizeWindow = output<void>();
    toggleMaximizeWindow = output<void>();
    closeWindow = output<void>();
    settingsRequested = output<void>();

    readonly isMacOS = navigator.platform.toLowerCase().includes("mac");
    readonly searchShortcut = this.isMacOS ? "⌘ K" : "Ctrl K";
    searchClass = "flex h-8 translate-y-px items-center gap-2.5 rounded-md border border-[rgba(122,153,181,0.24)] bg-[rgba(28,43,56,0.72)] py-0 pr-2.5 pl-3 text-[12px] text-[#aebdca] shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] hover:bg-[rgba(35,52,67,0.78)] [width:min(460px,calc(100vw-430px))]";
    searchInputClass = "h-full flex-1 leading-8 placeholder:text-[#8292a1]";

    readonly actions: Action[] = [
        {
            name: "Matrix Republic",
            description: "Matrix Republic",
            image: "assets/logo.png",
            action: () => this.openMatrixRepublic(),
        },
        {
            name: "Sponsor on GitHub",
            description: "Sponsor on GitHub",
            icon: "github",
            iconClass: "text-[14px]",
            action: () => this.openSponsorPage(),
        },
        {
            name: "Settings",
            description: "Settings",
            icon: "gear",
            iconClass: "text-[14px]",
            action: () => this.settingsRequested.emit(),
        },
    ];

    startWindowDrag(event: MouseEvent): void {
        if (event.button !== 0 || this.isInteractiveTarget(event.target)) {
            return;
        }

        this.dragStart.emit(event);
    }

    maximizeOnDoubleClick(event: MouseEvent): void {
        if (this.isInteractiveTarget(event.target)) {
            return;
        }

        this.toggleMaximizeWindow.emit();
    }

    private isInteractiveTarget(target: EventTarget | null): boolean {
        return target instanceof HTMLElement && target.closest("button, input, a, mtx-search-box") !== null;
    }

    openSponsorPage(): void {
        openUrl("https://github.com/sponsors/AndrewLang").catch(() => undefined);
    }

    openMatrixRepublic(): void {
        openUrl("https://matrixrepublic.net/").catch(() => undefined);
    }
}
