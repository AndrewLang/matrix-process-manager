import { Component, input, signal } from "@angular/core";
import { IconComponent } from "../icon/icon.component";

@Component({
    selector: "mtx-copy-button",
    imports: [IconComponent],
    template: `
        <button type="button" [class]="buttonClass()" [disabled]="!value()" (click)="copy($event)" [attr.aria-label]="label()" [title]="copied() ? 'Copied' : label()">
            <mtx-icon [name]="copied() ? 'check2' : 'copy'" />
        </button>
    `,
})
export class CopyButtonComponent {
    value = input.required<string>();
    label = input("Copy");
    buttonClass = input("grid size-6 place-items-center rounded bg-transparent text-(--muted) hover:bg-white/8 hover:text-[#e6f0fa] disabled:opacity-40");
    copied = signal(false);

    copy(event: MouseEvent): void {
        event.stopPropagation();
        const value = this.value();
        if (!value || this.copied()) {
            return;
        }

        navigator.clipboard.writeText(value).then(
            () => {
                this.copied.set(true);
                window.setTimeout(() => this.copied.set(false), 1200);
            },
            () => undefined,
        );
    }
}
