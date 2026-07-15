# Startup Apps

The Startup Apps screen shows the applications configured to launch when your
computer starts, and lets you inspect and edit their launch commands. Reducing
unnecessary startup items can make your computer boot faster and use fewer
resources.

## What You Can Do

- See the list of startup applications for your account.
- View details such as publisher, status, impact, startup type, and source.
- Inspect and edit the command an entry runs.

## Open Startup Apps

1. In the sidebar, select **Startup Apps**.
2. The list loads, sorted by name.

## Understanding the Details

Each entry includes information such as:

- **Name** and **Publisher**.
- **Status** — whether the entry is currently active.
- **Impact** — an indication of startup cost.
- **Startup type** and **Source** — where the entry comes from (for example, a
  registry key, a startup folder, or a login item).
- **Command** and **Path** — what runs and from where.

## Edit a Startup Command

1. Select the startup entry you want to change.
2. Edit its command.
3. Save the change.

> Editing a startup command changes what runs at login. An incorrect command can
> stop an app from launching at startup. Note the original value before changing
> it so you can restore it if needed.

## Where Entries Come From

- **Windows** — registry "Run" keys, the startup approval state, the per-user and
  common Startup folders, and packaged apps.
- **macOS** — Login item / LaunchAgent definitions. System-level agents are shown
  for reference and are not editable.

## Related Setting

Whether Prism itself starts with your computer is a separate option
(**Start with Windows**) on the [Settings](/prism/settings/general) screen, not an
entry you edit here.

## Recommended Workflow

1. Review the list and identify apps you do not need at startup.
2. Confirm the publisher and command before changing anything.
3. Make one change at a time and restart to verify the effect.

## Limitations

- System-level startup entries may be read-only.
- Available fields and editability depend on your platform and account
  permissions.

## Next Steps

- [Settings](/prism/settings/general)
- [Processes](/prism/monitoring/processes)
