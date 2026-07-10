import { ChangeDetectionStrategy, Component, input, output } from "@angular/core";

@Component({
    selector: "mtx-select",
    template: `
        <span class="select-shell">
            <select class="select-control" [value]="value()" [disabled]="disabled()" [attr.aria-label]="ariaLabel() || null" (change)="handleChange($event)">
                <ng-content />
            </select>
            <i class="bi bi-chevron-down select-icon"></i>
        </span>
    `,
    styles: [`
        :host {
            display: inline-block;
            min-width: max-content;
            height: 2rem;
            color: #dfe8f1;
            font-size: 12px;
        }

        .select-shell {
            position: relative;
            display: block;
            width: 100%;
            height: 100%;
        }

        .select-control {
            width: 100%;
            height: 100%;
            appearance: none;
            border: 1px solid var(--border);
            border-radius: 6px;
            background: rgba(10, 22, 33, 0.86);
            color: inherit;
            font: inherit;
            line-height: 1;
            outline: none;
            padding: 0 2.25rem 0 0.75rem;
        }

        .select-control:hover {
            border-color: rgba(23, 144, 255, 0.45);
        }

        .select-control:focus-visible {
            border-color: rgba(23, 144, 255, 0.72);
            box-shadow: 0 0 0 2px rgba(23, 144, 255, 0.14);
        }

        .select-control:disabled {
            cursor: not-allowed;
            opacity: 0.56;
        }

        .select-icon {
            pointer-events: none;
            position: absolute;
            top: 50%;
            right: 0.7rem;
            transform: translateY(-50%);
            color: #7ea1bb;
            font-size: 0.78em;
        }
    `],
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SelectComponent {
    value = input<string>("");
    disabled = input(false);
    ariaLabel = input("");
    valueChange = output<string>();

    handleChange(event: Event): void {
        this.valueChange.emit((event.target as HTMLSelectElement).value);
    }
}