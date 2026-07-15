# Overview

Prism is a desktop application for monitoring and managing your workstation. It
brings process monitoring, live performance charts, and a set of practical system
tools together in one window, so you can see what your computer is doing and act
on it without hunting through separate utilities.

![Prism logo](images/app.png)

## Who Prism Is For

Prism is designed for everyday power users, developers, and IT-minded people who
want to:

- Keep an eye on CPU, memory, disk, network, and GPU usage.
- Find and stop processes that are using too many resources.
- Clean up disk space and manage what launches at startup.
- Inspect open ports, discover network devices, and manage SSH keys.
- Work with Docker containers and images.
- Open a built-in terminal and quick shortcuts to native system tools.

## Supported Platforms

- Windows
- macOS

Some features are platform-specific. Where a tool depends on your operating
system, the relevant page notes it. In general, the deepest hardware detail and
the disk-cleanup targets are richest on Windows.

## Main Features

- **Dashboard** — an at-a-glance summary of system resources and activity.
- **Processes** — a live, sortable list of running processes with the ability to
  end a process.
- **Performance** — dedicated CPU, GPU, Memory, Network, and Disk monitors with
  history.
- **System Info** — hardware and operating-system details.
- **Startup Apps** — see and edit what runs when your computer starts.
- **Storage Cleanup** — scan for reclaimable space and clean it safely.
- **Ports** — see which ports are in use and which process owns them.
- **Network Devices** — discover devices on your local network.
- **SSH Keys** — list existing keys and generate new ones.
- **Docker** — manage containers and images (local or over SSH).
- **Command Center (Beta)** — an in-app terminal with optional command
  intelligence.
- **Settings** — configure behavior, appearance, and quick launchers for native
  tools.

## Why Use Prism

- One window instead of several separate system utilities.
- Live, continuously updated system data.
- Quick, direct actions (end a process, clean a cache, generate a key) rather
  than deep menu digging.
- Shortcuts to the native OS tools you already know.

## Typical Use Cases

- Your fan is spinning up — open **Processes** to find the culprit and end it.
- You are low on disk space — run **Storage Cleanup** to reclaim caches and temp
  files.
- A port is already in use — check **Ports** to find the owning process.
- You need a new deploy key — generate one in **SSH Keys** and copy the public
  key.
- You want to restart a container quickly — use the **Docker** screen.

## Next Steps

- [Install Prism](/prism/installation)
- [Follow the Quick Start](/prism/quick-start)
- [Explore the Dashboard](/prism/monitoring/dashboard)
