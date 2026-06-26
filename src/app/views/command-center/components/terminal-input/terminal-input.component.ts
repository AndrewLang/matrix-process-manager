import { Component, computed, input, output, signal } from "@angular/core";
import { CommandAutocompleteOption, CommandAutocompleteSuggestion } from "../../command-center.models";

@Component({
    selector: "mtx-terminal-input",
    templateUrl: "./terminal-input.component.html",
})
export class TerminalInputComponent {
    value = input.required<string>();
    disabled = input(false);
    suggestions = input<CommandAutocompleteSuggestion[]>([]);
    autocompleteOpen = input(false);
    autocompleteLoading = input(false);
    valueChange = output<string>();
    selectSuggestion = output<CommandAutocompleteSuggestion>();
    dismissAutocomplete = output<void>();
    submitInput = output<void>();

    activeIndex = signal(0);
    activeSuggestion = computed(() => this.suggestions()[this.activeIndex()]);

    updateValue(value: string): void {
        this.activeIndex.set(0);
        this.valueChange.emit(value);
    }

    chooseSuggestion(suggestion: CommandAutocompleteSuggestion): void {
        this.activeIndex.set(0);
        this.selectSuggestion.emit(suggestion);
    }

    setActiveIndex(index: number): void {
        this.activeIndex.set(index);
    }

    optionText(suggestion: CommandAutocompleteSuggestion): string {
        const option = suggestion.options[0];
        if (!option) {
            return "No options indexed";
        }
        return this.formatOption(option);
    }

    formatOption(option: CommandAutocompleteOption): string {
        return `${option.shortName ? `${option.shortName}, ` : ""}${option.name}${option.takesValue ? " <value>" : ""}`;
    }

    argumentText(suggestion: CommandAutocompleteSuggestion): string {
        const argument = suggestion.arguments[0];
        if (!argument) {
            return "No arguments indexed";
        }
        return `${argument.name}${argument.required ? " required" : " optional"}`;
    }

    exampleText(suggestion: CommandAutocompleteSuggestion): string {
        return suggestion.examples[0]?.commandLine ?? suggestion.commandLine;
    }

    handleKeydown(event: KeyboardEvent): void {
        const suggestions = this.suggestions();
        if (event.key === "Enter") {
            event.preventDefault();
            this.dismissAutocomplete.emit();
            this.submitInput.emit();
            return;
        }

        if (!this.autocompleteOpen() || suggestions.length === 0) {
            return;
        }

        if (event.key === "ArrowDown") {
            event.preventDefault();
            this.activeIndex.update((index) => (index + 1) % suggestions.length);
        } else if (event.key === "ArrowUp") {
            event.preventDefault();
            this.activeIndex.update((index) => (index - 1 + suggestions.length) % suggestions.length);
        } else if (event.key === "Tab") {
            event.preventDefault();
            this.chooseSuggestion(this.activeSuggestion() ?? suggestions[0]);
        } else if (event.key === "Escape") {
            event.preventDefault();
            this.dismissAutocomplete.emit();
        }
    }
}
