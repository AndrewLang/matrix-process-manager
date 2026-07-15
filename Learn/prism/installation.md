# Installation

This page explains how to install Prism, launch it for the first time, and keep
it up to date.

## Requirements

- **Windows** or **macOS**.
- On Windows, the WebView2 runtime is used to render the interface. It is
  included with current versions of Windows.
- Enough permissions to run a standard desktop application for your user account.

Some individual features rely on other software being present on your system:

- **Docker** features require Docker to be installed and available on your
  system `PATH` (or a reachable remote host over SSH).
- **SSH Keys** generation uses your system's SSH tooling.

## Install Prism

Prism is distributed as a standard desktop application for each platform.

### Windows

1. Obtain the Prism installer for Windows.
2. Run the installer and follow the prompts.
3. Launch **Prism** from the Start menu or desktop shortcut.

### macOS

1. Obtain the Prism application (`.app`, typically delivered in a `.dmg`).
2. Open the disk image and drag **Prism** to your **Applications** folder.
3. Launch **Prism** from **Applications** or Launchpad.

> The exact download location for released installers needs confirmation from the
> product owner. See [Facts that need confirmation](/prism/troubleshooting/common-issues).

## First Launch

When Prism starts:

1. The main window opens with the **Dashboard** selected.
2. Live system data begins to populate after the first refresh.
3. Use the sidebar on the left to move between screens.

The window uses a custom title bar. You can drag the title bar to move the
window, and double-click it to maximize or restore.

## Required Permissions

Prism reads system information and can perform actions such as ending a process
or cleaning files. These actions use your current user account's permissions.
Operations that affect protected system locations or processes may be limited by
the operating system if your account lacks the necessary rights.

The following behaviors are worth knowing:

- **Start with Windows** adds a startup entry for your user (Windows only).
- **Ending a process** uses the operating system's own termination mechanism.
- **Storage Cleanup** deletes files in the locations you select.

## Updates

Update delivery for released builds needs confirmation from the product owner.
When a new version is available, install it the same way as the original
installation for your platform.

## Uninstallation

- **Windows** — uninstall Prism from **Settings → Apps** (or **Add or remove
  programs**).
- **macOS** — quit Prism, then move the **Prism** app from **Applications** to
  the Trash.

## Recommended Initial Setup

1. Open **Settings** (the gear button in the title bar).
2. Under **General**, choose an **Update frequency** that suits you (see
   [Settings](/prism/settings/general)).
3. Decide whether to enable **Minimize to system tray** and **Start with
   Windows**.
4. Under **Tools**, enable the native tool shortcuts you want in the sidebar.

## Next Steps

- [Follow the Quick Start](/prism/quick-start)
- [Configure Settings](/prism/settings/general)
