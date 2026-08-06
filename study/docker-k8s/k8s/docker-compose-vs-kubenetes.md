

---

# Docker Compose vs Kubernetes

Docker Compose and Kubernetes solve similar problems but are used in different environments.

Docker Compose is mainly for local development.

Kubernetes is mainly for production environments and running applications at scale.

---

## Docker Compose (Development)

Docker Compose defines how multiple containers run together on a developer machine.

Example:

```yaml
services:
  frontend:
    image: frontend

  backend:
    image: backend

  database:
    image: postgres
````

This creates:

Developer Laptop

* Frontend Container
* Backend Container
* Database Container
```

Docker Compose does not have:

* Pods
* Nodes
* Clusters
* Kubernetes scheduling
* Self-healing
* Automatic scaling

---

## Kubernetes Deployment (Production)

The same application is usually split into separate Kubernetes resources.

Example:

```
Kubernetes Cluster

Frontend Deployment
        |
        v
    Frontend Pods
        |
        v
 Frontend Containers


Backend Deployment
        |
        v
    Backend Pods
        |
        v
 Backend Containers


Database StatefulSet
        |
        v
    Database Pod
        |
        v
 Database Container
```

Usually:

* Frontend runs in its own Pods
* Backend runs in its own Pods
* Database runs in its own Pod(s)

Each component can scale independently.

Example:

```
Frontend:
3 Pods


Backend:
10 Pods


Database:
1 Pod
```

---

# Docker Image Flow

Docker images are shared between development and Kubernetes.

The workflow:

```
Developer

    |
    v

Dockerfile

    |
    v

Docker Image

    |
    +----------------+
    |                |
    v                v

Docker Compose     Container Registry

(local dev)              |
                         v

                  Kubernetes Cluster
                         |
                         v

                       Pods
```

The same Docker image can run:

* Locally using Docker Compose
* In production using Kubernetes

---

# Project Structure Example

A common project keeps both configurations:

```
project/

|-------- frontend/
|          |---- Dockerfile
|
|-------- backend/
|          |--- Dockerfile
|
|-------- docker-compose.yml
|
|--------- k8s/
            |----- frontend-deployment.yaml
            |----- frontend-service.yaml
            |----- backend-deployment.yaml
            |----- backend-service.yaml
            |----- database-statefulset.yaml
            |----- database-service.yaml
```

---

# Mapping Docker Compose to Kubernetes

| Docker Compose                | Kubernetes                |
| ----------------------------- | ------------------------- |
| service                       | Deployment / StatefulSet  |
| container                     | container                 |
| image                         | image                     |
| compose network               | Kubernetes networking     |
| volume                        | Volume / PersistentVolume |
| environment variables         | ConfigMap / Secret        |
| multiple containers on laptop | Pods across Nodes         |
| docker compose up             | kubectl apply             |

---

# Why not use docker-compose in Kubernetes?

Docker Compose describes:

> "Run these containers together on one machine."

Kubernetes describes:

> "Run these applications across a cluster of machines and keep them healthy."

Kubernetes needs additional information that Docker Compose does not provide:

* Number of replicas
* Which node should run Pods
* Update strategy
* Health checks
* Storage rules
* Networking rules
* Scaling behavior

Therefore, production Kubernetes environments usually use separate Kubernetes YAML files.

```
```
