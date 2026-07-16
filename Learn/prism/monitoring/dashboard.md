# Dashboard

The Dashboard is Prism's home screen. It gives you a single, at-a-glance view of
how your workstation is performing right now, so you can spot problems quickly and
decide where to look next.

![The Prism Dashboard showing CPU, GPU, memory, disk, and network metrics](images/prism-dashboard.jpg)

## What You Can Do

- See summary metrics for CPU, GPU, memory, disk, and network.
- View small trend sparklines alongside each metric.
- Use it as a jumping-off point to the detailed monitoring screens.

## Open the Dashboard

1. Launch Prism.
2. The Dashboard is selected by default.
3. To return later, select **Dashboard** at the top of the sidebar.

## Understanding the Metrics

Each metric card shows a current value, a short detail line, and a small trend
line built from recent samples. The values refresh automatically.

- **CPU** — overall processor usage.
- **GPU** — graphics processor usage.
- **Memory** — memory in use.
- **Disk** — disk activity.
- **Network** — network throughput.

## Recommended Workflow

1. Open the Dashboard to check overall health.
2. If a metric looks high, open [Performance](/prism/monitoring/performance) for
   that resource.
3. If a specific app is responsible, switch to
   [Processes](/prism/monitoring/processes) to act on it.

## Limitations

- The Dashboard shows summary values. For per-adapter or per-drive detail, use
  [Performance](/prism/monitoring/performance).
- Some hardware detail is richer on Windows than on macOS.

## Next Steps

- [Performance](/prism/monitoring/performance)
- [Processes](/prism/monitoring/processes)
- [System Info](/prism/monitoring/system-info)
