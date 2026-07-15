# Network Devices

The Network Devices screen discovers devices on your local network and shows how
each was found and whether it is reachable. Use it to get a quick picture of
what is connected to your network.

## What You Can Do

- Scan the local network for devices.
- See each device's IP address and, where available, MAC address and hostname.
- See the network interface, discovery source, and reachability of each device.

## Open Network Devices

1. In the sidebar, select **Network Devices**.
2. Run a scan.
3. Review the discovered devices.

## Reading the Results

Each device can include:

- **IP address**.
- **MAC address** and **Hostname** — where available.
- **Interface** — the network interface the device was seen on.
- **Source** — how the device was discovered.
- **Reachable** — whether the device currently responds.

## Recommended Workflow

1. Run a scan while connected to the network you want to inspect.
2. Review the list to confirm expected devices are present.
3. Re-scan if you connect or disconnect devices.

## Limitations

- Discovery depends on your operating system's network stack and your active
  interfaces.
- Devices that do not respond may appear as unreachable or may not appear at all.
- Results reflect the moment the scan runs.

## Next Steps

- [Ports](/prism/tools/ports)
- [System Info](/prism/monitoring/system-info)
