import { Component, OnInit, computed, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { SshKeyGenerationRequest, SshKeyInfo } from "../../app.models";
import { DataGridColumn, DataGridComponent } from "../../components/data-grid/data-grid.component";
import { IconComponent } from "../../components/icon/icon.component";
import { SelectComponent } from "../../components/select/select.component";

@Component({
    selector: "mtx-ssh-keys-view",
    imports: [DataGridComponent, IconComponent, SelectComponent],
    templateUrl: "./ssh-keys-view.component.html",
})
export class SshKeysViewComponent implements OnInit {
    keys = signal<SshKeyInfo[]>([]);
    selectedName = signal("");
    loading = signal(false);
    generating = signal(false);
    error = signal("");
    actionMessage = signal("");
    newFileName = signal("id_ed25519_workstation");
    newKeyType = signal("ed25519");
    newComment = signal(this.defaultComment());
    columns: DataGridColumn<SshKeyInfo>[] = [
        { key: "name", label: "Name", width: 190, minWidth: 130, value: (key) => key.name, cellClass: () => "font-mono text-[#e6f0fa]" },
        { key: "keyType", label: "Type", width: 120, minWidth: 90, value: (key) => key.keyType },
        { key: "comment", label: "Comment", width: 220, minWidth: 140, value: (key) => key.comment },
        { key: "fingerprint", label: "Fingerprint", width: 360, minWidth: 220, value: (key) => key.fingerprint, cellClass: () => "font-mono text-(--muted)" },
        { key: "status", label: "Status", width: 120, minWidth: 96, value: (key) => key.hasPrivateKey ? "Pair" : "Public only" },
    ];
    rowKey = (key: SshKeyInfo) => key.name;

    selectedKey = computed(() => this.keys().find((key) => key.name === this.selectedName()) ?? this.keys()[0]);
    privateKeyCount = computed(() => this.keys().filter((key) => key.hasPrivateKey).length);
    publicOnlyCount = computed(() => this.keys().filter((key) => !key.hasPrivateKey).length);

    ngOnInit(): void {
        this.refresh();
    }

    refresh(): void {
        this.loading.set(true);
        this.error.set("");
        invoke<SshKeyInfo[]>("list_ssh_keys")
            .then((keys) => {
                this.keys.set(keys);
                const selected = keys.find((key) => key.name === this.selectedName()) ?? keys[0];
                this.selectedName.set(selected?.name ?? "");
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "SSH keys could not be loaded."))
            .finally(() => this.loading.set(false));
    }

    selectKey(key: SshKeyInfo): void {
        this.selectedName.set(key.name);
        this.actionMessage.set("");
    }

    setNewFileName(value: string): void {
        this.newFileName.set(value);
    }

    setNewKeyType(value: string): void {
        this.newKeyType.set(value);
    }

    setNewComment(value: string): void {
        this.newComment.set(value);
    }

    generateKey(): void {
        if (this.generating()) {
            return;
        }

        const request: SshKeyGenerationRequest = {
            fileName: this.newFileName().trim(),
            keyType: this.newKeyType(),
            comment: this.newComment().trim(),
        };

        if (!request.fileName || !request.comment) {
            this.error.set("File name and comment are required.");
            return;
        }

        this.generating.set(true);
        this.error.set("");
        this.actionMessage.set("");
        invoke<SshKeyInfo>("generate_ssh_key", { request })
            .then((key) => {
                this.actionMessage.set(`${key.name} generated.`);
                this.refresh();
                this.selectedName.set(key.name);
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "SSH key could not be generated."))
            .finally(() => this.generating.set(false));
    }

    copyPublicKey(key: SshKeyInfo | undefined): void {
        if (!key) {
            return;
        }

        navigator.clipboard.writeText(key.publicKey).then(
            () => this.actionMessage.set("Public key copied."),
            () => this.actionMessage.set("Public key could not be copied."),
        );
    }

    openKeyLocation(key: SshKeyInfo | undefined): void {
        const path = key?.privateKeyPath ?? key?.publicKeyPath;
        if (!path) {
            return;
        }

        revealItemInDir(path).catch(() => this.actionMessage.set("Key location could not be opened."));
    }

    modifiedText(key: SshKeyInfo): string {
        const modifiedAt = Number(key.modifiedAt ?? 0);
        return modifiedAt > 0 ? new Date(modifiedAt * 1000).toLocaleString() : "Unknown";
    }

    private defaultComment(): string {
        return "workstation-console@local";
    }
}
