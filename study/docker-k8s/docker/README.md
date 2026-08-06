# Core Concepts
```bash
Dockerfile
    |
    v
  docker build
    |
    v
  Image
    |
    v
docker run
    |
    v
 Container
     |
     |--- uses a Network
     |--- may use a Volume
     |--- may be pushed/pulled from a Registry
```
## Image

An image is an immutable, read-only template used to create containers. It contains everything an application needs to run, including the operating system libraries, runtime, dependencies, and application code.

Images are built from a `Dockerfile` and consist of multiple cached layers. Every time you rebuild an image, Docker reuses unchanged layers to speed up the build process.

**Think of it as:** A blueprint or snapshot for creating containers.

---

## Container

A container is a running (or stopped) instance of an image. It has its own isolated filesystem, processes, networking, and resource limits while sharing the host machine's kernel.

Containers are designed to be ephemeral-if a container is deleted, any data stored inside it is lost unless that data is stored in a volume or bind mount.

**Think of it as:** A running application created from an image.

---

## Dockerfile

A Dockerfile is a text file containing instructions for building an image. Each instruction creates a new image layer, allowing Docker to cache intermediate steps and avoid unnecessary rebuilds.

Common instructions include:

- `FROM` - Base image
- `WORKDIR` - Working directory
- `COPY` - Copy files into the image
- `RUN` - Execute commands during the build
- `EXPOSE` - Document which port the application listens on
- `CMD` - Default command when the container starts

---

## Registry

A registry is a service that stores and distributes Docker images.

You can push images to a registry after building them and pull them onto any machine with Docker installed.

Common registries include:

- Docker Hub
- GitHub Container Registry (GHCR)
- Amazon ECR
- Google Artifact Registry

---

## Volume

A volume provides persistent storage that exists independently of a container.

Without a volume, deleting a container also deletes any data stored inside it. Volumes allow databases, uploaded files, and other persistent data to survive container recreation.

Docker manages volumes separately from containers, making them easy to reuse and back up.

---

## Bind Mount

A bind mount maps a directory from the host machine directly into a container.

Unlike a volume, the files remain in their original location on the host. This is commonly used during development so code changes on your computer are immediately visible inside the container.

---

## Network

Docker networks allow containers to communicate with each other while remaining isolated from unrelated containers.

By default, containers connected to the same Docker network can communicate using their container names as hostnames, making it easy for services like web servers and databases to interact.

---

## Docker Engine

Docker Engine is the core software that builds images, creates containers, manages networks, and handles volumes.

The Docker CLI (`docker ...`) sends commands to the Docker daemon, which performs the requested operations.
