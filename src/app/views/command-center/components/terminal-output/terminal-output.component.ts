import { NgClass } from "@angular/common";
import { AfterViewChecked, Component, ElementRef, input, viewChild } from "@angular/core";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { TerminalLine, TerminalSegment, TerminalTextPart } from "../../command-center.models";

@Component({
    selector: "mtx-terminal-output",
    imports: [NgClass],
    templateUrl: "./terminal-output.component.html",
    host: { class: "block h-full min-h-0 min-w-0" },
})
export class TerminalOutputComponent implements AfterViewChecked {
    lines = input.required<TerminalLine[]>();
    maxRenderedLines = input(1200);
    viewport = viewChild<ElementRef<HTMLElement>>("viewport");
    private partCache = new WeakMap<TerminalSegment, TerminalTextPart[]>();
    private lastScrollHeight = 0;

    visibleLines(): TerminalLine[] {
        return this.lines().slice(-this.maxRenderedLines());
    }

    lineText(line: TerminalLine): string {
        return line.segments.map((segment) => segment.text).join("");
    }

    segmentParts(segment: TerminalSegment): TerminalTextPart[] {
        const cached = this.partCache.get(segment);
        if (cached) {
            return cached;
        }

        const matcher = /(https?:\/\/[^\s]+)|((?:[A-Za-z]:\\|~\/|\/)[^\s<>|"']+)/g;
        const parts: TerminalTextPart[] = [];
        let index = 0;
        for (const match of segment.text.matchAll(matcher)) {
            const text = match[0];
            const start = match.index ?? 0;
            if (start > index) {
                parts.push({ text: segment.text.slice(index, start), kind: "text" });
            }
            parts.push({ text, kind: text.startsWith("http://") || text.startsWith("https://") ? "url" : "path" });
            index = start + text.length;
        }
        if (index < segment.text.length) {
            parts.push({ text: segment.text.slice(index), kind: "text" });
        }
        this.partCache.set(segment, parts);
        return parts;
    }

    openPart(part: TerminalTextPart): void {
        if (part.kind === "url") {
            openUrl(part.text).catch(() => undefined);
        } else if (part.kind === "path") {
            revealItemInDir(part.text).catch(() => undefined);
        }
    }

    ngAfterViewChecked(): void {
        const viewport = this.viewport()?.nativeElement;
        if (viewport && viewport.scrollHeight !== this.lastScrollHeight) {
            this.lastScrollHeight = viewport.scrollHeight;
            viewport.scrollTop = viewport.scrollHeight;
        }
    }
}
