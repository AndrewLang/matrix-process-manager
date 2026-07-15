# Command Center

Command Center is Prism's built-in terminal. It lets you open real shell sessions
inside the app and, optionally, use command intelligence for autocomplete
suggestions and command history. This screen is labeled **Beta**.

## What You Can Do

- Open a terminal session using your chosen shell.
- Type commands and see live output.
- Resize the terminal.
- Optionally enable command intelligence for suggestions and history.

## Open Command Center

1. In the sidebar, select **Console (Beta)**.
2. A terminal session starts (or you start one), using your configured shell.
3. Type commands and interact as you would in a normal terminal.

The default shell and terminal appearance come from your Terminal settings. See
[Settings](/prism/settings/general).

## Terminal Behavior

- Output appears live as commands run.
- You can resize the terminal to fit your work.
- Sessions run real shell processes on your computer.

> Sessions are not saved between app restarts. When you close Prism, open terminal
> sessions end and their history within the session is not restored on the next
> launch.

## Command Intelligence (Optional)

Command intelligence provides autocomplete suggestions and records command
execution history. It is **off by default**.

To use it:

1. Open [Settings](/prism/settings/general) and go to **Terminal**.
2. Enable command intelligence.
3. Optionally configure the indexing cadence under **Indexing** (Manual, Startup,
   Hourly, or Daily).

When enabled, Prism can index installed command-line tools to power suggestions,
and can keep a history of commands you run.

## Terminal Appearance and Shell

Configure the following in **Settings → Terminal**:

- Default shell (System default, PowerShell, CMD, zsh, or bash).
- Font family and size.
- Cursor style (Block, Bar, or Underline).
- Opacity.
- Theme (Matrix, Midnight, or Slate).
- History size.
- Autocomplete delay (used with command intelligence).

## Recommended Workflow

1. Set your preferred shell and theme in Settings.
2. Open Command Center and work as you would in a normal terminal.
3. If you want suggestions, enable command intelligence and choose an indexing
   schedule.

## Limitations

- Command Center is a **Beta** feature; behavior may change.
- Terminal sessions are not persisted across app restarts.
- The available shells depend on what is installed on your system.

## Next Steps

- [Settings](/prism/settings/general)
- [SSH Keys](/prism/tools/ssh-keys)
