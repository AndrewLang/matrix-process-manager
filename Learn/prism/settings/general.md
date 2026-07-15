# Settings

The Settings screen controls how Prism behaves and looks, and provides shortcuts
to native system tools. Open it from the **gear** button in the title bar.

## Open Settings

1. Select the **Settings** (gear) button in the title bar.
2. Choose a category from the list: **General**, **Terminal**, **Indexing**,
   **Storage**, **Tools**, or **About**.

## General

General settings control core behavior.

- **Start with Windows** — launch Prism automatically when Windows starts.
  Windows only.
- **Minimize to system tray** — minimize to the system tray instead of the
  taskbar. With this on, closing to the tray keeps Prism running in the
  background; click the tray icon to bring the window back.
- **Confirm before killing processes** — show a confirmation dialog before ending
  a process. Leaving this on is safer.
- **Update frequency** — how often live data refreshes:
  - **High** — refresh every second.
  - **Normal** — refresh every 2 seconds.
  - **Low** — refresh every 5 seconds.
  - **Paused** — stop automatic updates.
- **Language** and **Date & time format** — display preferences.

Choosing a slower **Update frequency** (or **Paused**) reduces background work,
which can help on battery power.

## Terminal

Terminal settings apply to [Command Center](/prism/console/command-center):

- **Default shell** — System default, PowerShell, CMD, zsh, or bash.
- **Font family** and **Font size**.
- **Cursor style** — Block, Bar, or Underline.
- **Opacity**.
- **Theme** — Matrix, Midnight, or Slate.
- **History size**.
- **Command intelligence** — enable autocomplete suggestions and history.
- **Autocomplete delay** — how long to wait before showing suggestions.

## Indexing

Indexing settings control how often Prism refreshes its command knowledge used by
command intelligence:

- **Manual** — only index when requested.
- **Startup** — index once when the app starts.
- **Hourly** — refresh every hour.
- **Daily** — refresh once per day.

## Storage

Storage settings control where command knowledge data is stored.

## Tools

The Tools category toggles the native tool shortcuts that appear in the sidebar.
Enabling a tool adds its shortcut; selecting the shortcut opens the corresponding
system utility. Labels depend on your platform:

- **Task Manager** / **Activity Monitor**
- **System Settings**
- **Disk Manager** / **Disk Utility**
- **Terminal**
- **Env Variables** / **Environment**
- **Snipping Tool** / **Screenshot**

## About

The About category shows product and version information and a link to the
website.

## Recommended Workflow

1. Set your **Update frequency** to match how closely you want to watch the
   system.
2. Decide on tray and startup behavior.
3. Configure the Terminal to your taste if you use Command Center.
4. Enable the native tool shortcuts you use most.

## Notes and Limitations

- **Start with Windows** applies to Windows.
- Settings are stored locally for the app on your computer.

## Next Steps

- [Command Center](/prism/console/command-center)
- [Overview](/prism/overview)
