import { Component, OnInit, computed, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { NetworkDevice, NetworkDeviceScan } from "../../app.models";
import { DataGridColumn, DataGridComponent } from "../../components/data-grid/data-grid.component";
import { IconComponent } from "../../components/icon/icon.component";
import { SelectComponent } from "../../components/select/select.component";

type DeviceStateFilter = "all" | "reachable" | "local";

@Component({
    selector: "mtx-network-devices-view",
    imports: [DataGridComponent, IconComponent, SelectComponent],
    templateUrl: "./network-devices-view.component.html",
})
export class NetworkDevicesViewComponent implements OnInit {
    scan = signal<NetworkDeviceScan | undefined>(undefined);
    selectedKey = signal("");
    query = signal("");
    stateFilter = signal<DeviceStateFilter>("all");
    loading = signal(false);
    error = signal("");
    actionMessage = signal("");
    columns: DataGridColumn<NetworkDevice>[] = [
        { key: "ipAddress", label: "IP Address", width: 160, minWidth: 120, value: (device) => device.ipAddress, cellClass: () => "font-mono text-[#e6f0fa]" },
        { key: "hostname", label: "Host", width: 180, minWidth: 120, value: (device) => device.hostname },
        { key: "macAddress", label: "MAC Address", width: 180, minWidth: 130, value: (device) => device.macAddress, cellClass: () => "font-mono text-(--muted)" },
        { key: "interfaceName", label: "Interface", width: 190, minWidth: 130, value: (device) => device.interfaceName },
        { key: "state", label: "State", width: 120, minWidth: 90, value: (device) => device.state, iconClass: (device) => device.reachable ? "bi-circle-fill" : "bi-circle", iconColorClass: (device) => device.reachable ? "text-[#00b8a9]" : "text-[#7ea1bb]" },
        { key: "source", label: "Source", width: 150, minWidth: 110, value: (device) => device.source },
    ];
    rowKey = (device: NetworkDevice) => device.ipAddress;

    devices = computed(() => this.scan()?.devices ?? []);
    filteredDevices = computed(() => this.filterDevices());
    selectedDevice = computed(() => this.filteredDevices().find((device) => device.ipAddress === this.selectedKey()) ?? this.filteredDevices()[0]);
    reachableCount = computed(() => this.devices().filter((device) => device.reachable).length);
    localCount = computed(() => this.devices().filter((device) => device.source === "Local adapter").length);
    macCount = computed(() => this.devices().filter((device) => device.macAddress).length);

    ngOnInit(): void {
        this.refresh();
    }

    refresh(): void {
        this.loading.set(true);
        this.error.set("");
        this.actionMessage.set("");
        invoke<NetworkDeviceScan>("get_network_device_scan")
            .then((scan) => {
                this.scan.set(scan);
                const selected = scan.devices.find((device) => device.ipAddress === this.selectedKey()) ?? scan.devices[0];
                this.selectedKey.set(selected?.ipAddress ?? "");
                this.actionMessage.set(`${scan.devices.length} network devices found.`);
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Network device scan failed."))
            .finally(() => this.loading.set(false));
    }

    selectDevice(device: NetworkDevice): void {
        this.selectedKey.set(device.ipAddress);
        this.actionMessage.set("");
    }

    setQuery(value: string): void {
        this.query.set(value);
    }

    setStateFilter(filter: DeviceStateFilter): void {
        this.stateFilter.set(filter);
    }

    copyAddress(device: NetworkDevice | undefined): void {
        if (!device) {
            return;
        }

        navigator.clipboard.writeText(device.ipAddress).then(
            () => this.actionMessage.set("IP address copied."),
            () => this.actionMessage.set("IP address could not be copied."),
        );
    }

    scannedAtText(): string {
        const scannedAt = Number(this.scan()?.scannedAt ?? 0);
        return scannedAt > 0 ? new Date(scannedAt * 1000).toLocaleTimeString() : "Not scanned";
    }

    private filterDevices(): NetworkDevice[] {
        const query = this.query().trim().toLowerCase();
        const state = this.stateFilter();

        return this.devices()
            .filter((device) => state === "all" || state === "reachable" && device.reachable || state === "local" && device.source === "Local adapter")
            .filter((device) => {
                if (!query) {
                    return true;
                }

                return [
                    device.ipAddress,
                    device.macAddress ?? "",
                    device.hostname ?? "",
                    device.interfaceName,
                    device.state,
                    device.source,
                ].some((value) => value.toLowerCase().includes(query));
            });
    }
}