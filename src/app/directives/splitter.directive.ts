import { Directive, ElementRef, HostListener, inject, input, output, Renderer2 } from "@angular/core";

@Directive({
  selector: "[appSplitter]",
})
export class SplitterDirective {
  min = input(120);
  max = input(360);
  widthChange = output<number>();

  private readonly elementRef = inject<ElementRef<HTMLElement>>(ElementRef);
  private readonly renderer = inject(Renderer2);
  private startX = 0;
  private startWidth = 0;
  private dragging = false;

  @HostListener("mousedown", ["$event"])
  startDrag(event: MouseEvent): void {
    event.preventDefault();
    this.dragging = true;
    this.startX = event.clientX;
    this.startWidth = this.elementRef.nativeElement.previousElementSibling?.getBoundingClientRect().width ?? this.min();
    this.renderer.addClass(document.body, "select-none");
    this.renderer.addClass(document.body, "cursor-col-resize");
  }

  @HostListener("document:mousemove", ["$event"])
  drag(event: MouseEvent): void {
    if (!this.dragging) {
      return;
    }

    this.widthChange.emit(Math.min(this.max(), Math.max(this.min(), this.startWidth + event.clientX - this.startX)));
  }

  @HostListener("document:mouseup")
  stopDrag(): void {
    if (!this.dragging) {
      return;
    }

    this.dragging = false;
    this.renderer.removeClass(document.body, "select-none");
    this.renderer.removeClass(document.body, "cursor-col-resize");
  }
}
