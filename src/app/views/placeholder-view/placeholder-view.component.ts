import { Component, inject } from "@angular/core";
import { ActivatedRoute } from "@angular/router";

@Component({
    selector: "mtx-placeholder-view",
    templateUrl: "./placeholder-view.component.html",
})
export class PlaceholderViewComponent {
    route = inject(ActivatedRoute);
    title = this.route.snapshot.data["title"] as string;
}