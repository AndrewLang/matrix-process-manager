# Docker

The Docker screen lets you view and manage Docker containers and images from
inside Prism. You can work with your local Docker or connect to a remote host
over SSH. This screen appears in the sidebar only when Docker is detected.

![The Prism Docker screen showing containers and images](images/prism-docker.jpg)

## What You Can Do

- See whether Docker is installed and running.
- View containers and images.
- Start, stop, restart, and remove containers.
- Remove images.
- View a container's logs and inspect details.
- List images from a registry.
- Connect to a remote Docker host over SSH.

## Open the Docker Screen

1. Ensure Docker is installed and available (locally or on a reachable remote
   host).
2. In the sidebar, select **Docker**. If Docker is detected, the dashboard loads.

If Docker is not detected, the screen reports that it is unavailable. See
[Troubleshooting](#troubleshooting).

## The Dashboard

The dashboard shows:

- **Availability** — whether Docker is installed and running, with version
  information.
- **Containers** — name, image, state, status, ports, and creation time.
- **Images** — repository, tag, size, and creation time.

## Manage Containers

1. Locate the container in the list.
2. Choose an action: **Start**, **Stop**, **Restart**, or **Remove**.
3. The list refreshes to reflect the new state.

> Removing a container is destructive. Any data that is not stored in a volume or
> bind mount is lost when the container is removed.

## View Logs and Inspect

- **Logs** — view a container's recent log output.
- **Inspect** — view detailed configuration for a container.

## Remove an Image

1. Locate the image in the list.
2. Remove it.

> Removing an image deletes it. Containers or workflows that depend on that image
> will need to pull or rebuild it again.

## Work With a Remote Host

1. Provide the remote Docker host (an SSH target).
2. Prism runs Docker commands against that host over SSH.

For this to work, the SSH target must be reachable and have Docker available.

## List Registry Images

You can list images available in a registry to see repositories and their tags.

## Recommended Workflow

1. Confirm Docker is running (check the availability indicator).
2. Use the container actions for day-to-day lifecycle management.
3. Check logs before removing or restarting a container that is misbehaving.

## Limitations

- Docker features require the Docker CLI to be installed and on your system
  `PATH`, or a reachable remote host with Docker available.
- Remote management requires working SSH access to the target.

## Troubleshooting

- **Docker shows as unavailable** — confirm Docker is installed and running and
  that the Docker command is on your `PATH`.
- **A remote host does not connect** — confirm the SSH target is correct and
  reachable and that Docker is installed on it.

## Next Steps

- [Command Center](/prism/console/command-center)
- [Processes](/prism/monitoring/processes)
