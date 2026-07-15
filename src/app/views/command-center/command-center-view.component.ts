import { Component, OnDestroy, OnInit, inject } from "@angular/core";
import { IconComponent } from "../../components/icon/icon.component";
import { SelectComponent } from "../../components/select/select.component";
import { TerminalShell } from "./command-center.models";
import { TerminalInputComponent } from "./components/terminal-input/terminal-input.component";
import { TerminalOutputComponent } from "./components/terminal-output/terminal-output.component";
import { TerminalService } from "./services/terminal.service";
import { CommandCenterViewModel } from "./view-models/command-center.viewmodel";

@Component({
    selector: "mtx-command-center-view",
    imports: [IconComponent, SelectComponent, TerminalInputComponent, TerminalOutputComponent],
    providers: [CommandCenterViewModel, TerminalService],
    templateUrl: "./command-center-view.component.html",
})
export class CommandCenterViewComponent implements OnInit, OnDestroy {
    vm = inject(CommandCenterViewModel);

    ngOnInit(): void {
        this.vm.start().catch(() => undefined);
    }

    ngOnDestroy(): void {
        this.vm.stop().catch(() => undefined);
    }

    setShell(value: string): void {
        this.vm.setShell(value as TerminalShell);
    }

    importHistory(event: Event): void {
        const input = event.target as HTMLInputElement;
        this.vm.importHistory(input.files?.[0]).catch(() => undefined);
        input.value = "";
    }
}
