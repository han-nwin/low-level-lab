# Docker Compose

## What is Docker Compose?

Docker Compose is a tool for defining and running multi-container Docker applications.

A normal Docker workflow focuses on running individual containers:

```
Dockerfile
    |
    v
Docker Image
    |
    v
docker run
    |
    v
Container
```

Docker Compose manages applications made of multiple containers:

```
Application

    |
    +-------------+
    |             |
 Backend       Database
 Container     Container

    |
    |
  Redis
 Container
```

Instead of manually running multiple `docker run` commands, Compose allows you to describe the entire application stack in a YAML file:

```
docker-compose.yml
```

Then start everything with:

```bash
docker compose up
```

---

# Why use Docker Compose?

Docker Compose is commonly used for:

- Local development environments
- Running applications with dependencies
- Testing environments
- Multi-service applications

It manages:

- Multiple containers
- Container networking
- Environment variables
- Volumes
- Service configuration

---

# docker-compose.yml

A Compose file defines the services that make up an application.

Example:

```yaml
services:

  api:
    build: .
    ports:
      - "8080:8080"

  database:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: password
```

This creates:

```
api container

      |
      |
      v

database container
```

---

# Compose Concepts

## Service

A service defines a container that Compose should create and manage.

Example:

```yaml
services:
  backend:
    image: my-backend
```

Each service becomes a container when the application starts.

---

## Image

A service can use an existing image:

```yaml
database:
  image: postgres:16
```

or build an image from a Dockerfile:

```yaml
backend:
  build: .
```

When using `build`, Compose runs the equivalent of:

```bash
docker build
```

to create the image.

---

## Container

A container is an instance of a service.

Example:

```yaml
services:
  database:
    image: postgres
```

creates:

```
postgres image

      |
      v

postgres container
```

---

## Networking

Docker Compose automatically creates a network for the application.

Services can communicate using service names instead of IP addresses.

Example:

```yaml
services:

  backend:
    build: .

  database:
    image: postgres
```

The backend connects using:

```
database:5432
```

not:

```
localhost:5432
```

because `localhost` inside the backend container refers to the backend container itself.

---

## Volumes

Volumes provide persistent storage.

Example:

```yaml
services:

  database:
    image: postgres
    volumes:
      - postgres-data:/var/lib/postgresql/data


volumes:
  postgres-data:
```

Without volumes:

```
Container deleted

      |
      v

Data deleted
```

With volumes:

```
Container deleted

      |
      v

Data remains
```

Commonly used for:

- Databases
- Uploaded files
- Persistent application data

---

## Environment Variables

Environment variables allow configuration to be passed into containers.

Example:

```yaml
services:

  backend:
    environment:
      DATABASE_HOST: database
      DATABASE_PORT: 5432
```

Common uses:

- Database configuration
- API keys
- Runtime settings

---

# Build vs Image

A service can either build an image or use an existing one.

## Build from Dockerfile

```yaml
services:

  api:
    build: .
```

Flow:

```
Source Code

    |
    v

Dockerfile

    |
    v

Docker Image

    |
    v

Container
```

---

## Use existing image

```yaml
services:

  database:
    image: postgres:16
```

Flow:

```
Docker Registry

       |
       v

Docker Image

       |
       v

Container
```

---

# Example Application Stack

A common backend setup:

```
Application

+----------------+
|                |
|  C++ Backend   |
|                |
+----------------+

        |
        |

+----------------+
|   PostgreSQL   |
+----------------+

        |
        |

+----------------+
|     Redis      |
+----------------+
```

Example:

```yaml
services:

  backend:
    build: .
    ports:
      - "8080:8080"
    depends_on:
      - database
      - redis


  database:
    image: postgres:16


  redis:
    image: redis:7
```

Starting the application:

```bash
docker compose up
```

creates all three containers and connects them together.

---

# Docker Compose vs Kubernetes

## Docker Compose

Best for:

- Local development
- Small environments
- Testing
- Running multiple services on one machine

Example:

```
Developer Laptop

    |
    |
 docker compose

    |
    +-- Backend
    +-- Database
    +-- Redis
```

---

## Kubernetes

Best for:

- Production deployments
- Multiple machines
- Large-scale applications

Example:

```
Kubernetes Cluster

    |
    +-- Node 1
    |     |
    |    Pods
    |
    +-- Node 2
          |
         Pods
```

---

# Typical Workflow

Develop locally:

```
Dockerfile
      |
      v
docker compose up
      |
      v
Multiple containers running locally
```

Deploy:

```
Docker Image
      |
      v
Container Registry
      |
      v
Kubernetes
```
