# Processes

The Processes screen shows the applications and background processes running on
your computer, with live resource usage. Use it to understand what is consuming
CPU, memory, disk, or network, and to end processes that are misbehaving.

## What You Can Do

- View a live list of running processes.
- See each process's name, publisher, PID, status, user, and resource usage.
- Sort and resize columns to focus on what matters.
- End (terminate) a selected process.

## Open the Processes Screen

1. In the sidebar, select **Processes**.
2. The list populates and refreshes automatically.

## Reading the List

Processes are grouped to help you tell foreground apps from background work:

- **Apps** — foreground applications.
- **Background** — background processes.
- **Windows** — operating-system processes (grouping varies by platform).

Each row includes CPU, GPU, memory, disk, and network figures so you can compare
usage at a glance. Where available, Prism shows the application's own icon.

## End a Process

1. Select the process you want to stop.
2. Trigger the end/terminate action.
3. If confirmation is enabled, confirm in the dialog.

> Ending a process stops it immediately. Any unsaved work in that application may
> be lost. Ending critical system processes can make your system unstable.

### Confirmation Setting

By default, Prism asks before ending a process
(**Confirm before killing processes**). You can turn this off in
[Settings](/prism/settings/general), but leaving it on is safer.

## Main Options

- **Sorting** — click a column header to sort by that column (for example, sort
  by CPU to find the heaviest process).
- **Column resizing** — drag a column border to widen or narrow it.

## Recommended Workflow

1. Sort by CPU or memory to find the heaviest process.
2. Confirm it is the app you think it is (check the name and publisher).
3. Try closing it normally from the application first.
4. If it is unresponsive, end it from Prism and confirm.

## Limitations

- Terminating a process that your account cannot control may fail or be blocked by
  the operating system.
- Resource figures are sampled at the current refresh interval, so very short
  spikes may not always be visible.

## Troubleshooting

- **A process will not end** — you may lack the permissions required to stop it,
  or it may be a protected system process.
- **The list seems frozen** — check your **Update frequency**; if it is set to
  **Paused**, automatic updates are stopped
  ([Settings](/prism/settings/general)).

## Next Steps

- [Performance](/prism/monitoring/performance)
- [Ports](/prism/tools/ports)
