# Ports

The Ports screen shows the network ports in use on your computer and, where
available, which process owns each one. Use it to find out what is listening on a
port or to track down a port conflict.

![The Prism Ports screen showing ports in use and their owning processes](images/prism-ports.jpg)

## What You Can Do

- Scan for ports currently in use.
- See the protocol, local address and port, and connection state.
- See the owning process (PID, name, and path) where available.
- See remote address and port for active connections where available.

## Open the Ports Screen

1. In the sidebar, select **Ports**.
2. Run a scan to list ports in use.
3. Review the results, including the owning process.

## Reading the Results

Each entry can include:

- **Protocol** — for example TCP or UDP.
- **Local address** and **Local port**.
- **State** — the connection state.
- **Remote address** and **Remote port** — for active connections.
- **Process** — the PID, name, and path of the owner.

## Recommended Workflow

1. Run a scan and locate the port you are interested in.
2. Note the owning process.
3. If you need to free the port, switch to
   [Processes](/prism/monitoring/processes) to end that process.

> Ending a process to free a port stops that application immediately. Make sure it
> is the correct process and that stopping it is safe.

## Limitations

- The owning-process details depend on your platform and permissions and may not
  always be available.
- A scan reflects the moment it runs; re-scan to see current state.

## Troubleshooting

- **No owning process is shown** — your account may lack the permissions needed to
  read ownership for that connection.

## Next Steps

- [Processes](/prism/monitoring/processes)
- [Network Devices](/prism/tools/network-devices)
