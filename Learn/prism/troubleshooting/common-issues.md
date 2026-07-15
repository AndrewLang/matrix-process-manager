# Troubleshooting

This page covers common questions and issues when using Prism, along with facts
that still need confirmation from the product owner.

## The Window Does Not Appear Right Away

Prism shows its window after it finishes preparing on launch, so a brief delay is
normal. If the window never appears, close Prism fully and start it again.

## A Screen Shows No Data or an Error

Prism reads live system information to populate its screens.

- Make sure you launched the full Prism desktop app.
- Some features depend on your platform (see below).

## A Feature Seems to Do Nothing on My Platform

Several features are platform-specific by design:

- **Start with Windows** applies to Windows only.
- **Storage Cleanup** targets are primarily Windows locations.
- **Startup Apps** reads Windows startup entries or macOS login items depending on
  your platform.
- The most detailed hardware metrics are available on Windows.

If a feature appears inactive, confirm it is supported on your operating system.

## I Cannot End a Process

- You may not have permission to stop that process.
- It may be a protected system process.

Try closing the application normally first. See
[Processes](/prism/monitoring/processes).

## Docker Shows as Unavailable

- Confirm Docker is installed and running.
- Confirm the Docker command is available on your system `PATH`.
- For a remote host, confirm the SSH target is correct and reachable and that
  Docker is installed on it.

See [Docker](/prism/tools/docker).

## My Terminal Session Disappeared

Command Center sessions are not saved between app restarts. When Prism closes,
open sessions end. See [Command Center](/prism/console/command-center).

## My Settings or Window Position Reset

Prism stores your settings and window position locally for the app. Clearing that
data, or a corrupted value, resets them to defaults.

## Command Suggestions Do Not Appear

Command intelligence is off by default and relies on an index.

1. Enable command intelligence in **Settings → Terminal**.
2. Choose an indexing schedule in **Settings → Indexing**, or index manually.

See [Command Center](/prism/console/command-center).

## Facts That Need Confirmation

The following details could not be fully verified from the application and should
be confirmed by the product owner:

- **Download location** for released installers on each platform.
- **Update delivery** mechanism (whether updates are automatic or manual).
- **Displayed version number.** The About screen shows version **1.0.0**, while
  the application's build files indicate **0.1.0**. Confirm which is correct.
- **Language** and **Date & time format** options under General settings — confirm
  whether these currently change the application's language and formatting.

## Next Steps

- [Overview](/prism/overview)
- [Installation](/prism/installation)
- [Settings](/prism/settings/general)
