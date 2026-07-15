import { Component, computed, input } from "@angular/core";

@Component({
    selector: "mtx-icon",
    template: "",
    host: {
        "[class]": "iconClass()",
    },
})
export class IconComponent {
    name = input.required<string>();

    readonly iconClass = computed(() => {
        const raw = this.name();
        const name = raw.startsWith("bi-") ? raw.slice(3) : raw;
        return `bi bi-${name}`;
    });
}
