# Storage Cleanup

The Storage Cleanup screen scans your system for space you can safely reclaim —
temporary files, caches, and other reclaimable locations — and lets you remove
them. Use it to free up disk space quickly.

## What You Can Do

- Scan your drives for reclaimable space.
- Review standard cleanup targets and their sizes.
- Review developer and package-manager caches ("usage insights"), each flagged
  for how safe it is to clean.
- Clean the items you select and see how much space was freed.

## Open Storage Cleanup

1. In the sidebar, select **Storage Cleanup**.
2. Run a scan.
3. Review the results grouped into volumes, cleanup targets, and usage insights.

## Understanding the Results

- **Volumes** — your drives with total and free space.
- **Cleanup targets** — standard reclaimable locations (for example, temporary
  files, Recycle Bin, and system caches), each with its current size.
- **Usage insights** — developer and package caches (such as npm, pnpm, pip,
  NuGet, Cargo, Gradle, and similar), each labeled with how safe it is to clean.

## Clean Up Space

1. Review the scan results.
2. Select the targets or insights you want to remove.
3. Start the cleanup.
4. Prism reports how much space was freed.

> Cleaning permanently deletes the files in the selected locations. Review your
> selection first. Clearing a developer cache is safe but means those tools will
> re-download or rebuild cached data the next time they need it.

## Main Options

- **Standard targets vs. usage insights** — standard targets are common
  system/temporary locations; usage insights are development caches. Insights
  carry a safety label to help you decide.

## Recommended Workflow

1. Run a scan and start with the clearly safe items (temporary files, caches).
2. Review larger targets individually before removing them.
3. Re-run the scan afterward to confirm the space was reclaimed.

## Limitations

- Cleanup targets are primarily oriented to Windows locations (for example,
  Recycle Bin and Windows temporary/update caches). Availability varies by
  platform.
- Files removed by cleanup are deleted and are not recoverable from Prism.

## Troubleshooting

- **A location shows zero size or is missing** — that path may not exist on your
  system, which is normal.
- **Cleanup did not free the expected space** — some files may have been in use;
  try again after closing related applications.

## Next Steps

- [System Info](/prism/monitoring/system-info)
- [Settings](/prism/settings/general)
