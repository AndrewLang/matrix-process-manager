# Performance

The Performance screen provides dedicated monitors for each major system
resource, with recent history so you can see trends rather than just the current
value.

## What You Can Do

- Switch between CPU, GPU, Memory, Network, and Disk monitors.
- View current usage together with a rolling history.
- See per-adapter and per-drive detail where your hardware and platform provide
  it.

## Open Performance

1. In the sidebar, select **Performance**.
2. The CPU monitor opens by default.
3. Use the resource tabs to switch between monitors.

## The Monitors

- **CPU** — overall and detailed processor activity.
- **GPU** — graphics adapter usage, including engine breakdown where available.
- **Memory** — memory usage and breakdown.
- **Network** — throughput per network adapter.
- **Disk** — activity per drive.

Each monitor keeps a rolling window of recent samples so you can watch how usage
changes over time.

## Recommended Workflow

1. Open the monitor for the resource you are investigating.
2. Watch the history to distinguish a brief spike from sustained load.
3. If a specific process is responsible, switch to
   [Processes](/prism/monitoring/processes).

## Limitations

- The amount of hardware detail varies by platform. The richest breakdowns (for
  example, GPU engines, disk activity percentages, and detailed memory pools) are
  available on Windows; some fields may be empty on macOS.

## Next Steps

- [Dashboard](/prism/monitoring/dashboard)
- [System Info](/prism/monitoring/system-info)
