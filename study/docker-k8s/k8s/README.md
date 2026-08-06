# Kubernetes

## What is Kubernetes?

Kubernetes (K8s) is an open-source container orchestration platform used to deploy, manage, and scale containerized applications.

Docker solves the problem of packaging an application into a container.

Kubernetes solves the problem of running many containers reliably across multiple machines.

Kubernetes manages:

- Container deployment
- Scaling
- Networking
- Service discovery
- Storage
- Configuration
- Application health
- Rolling updates
- Self-healing

---

# Why use Kubernetes?

Running containers manually works for small projects, but becomes difficult when applications grow.

Kubernetes helps by automatically handling:

## Scaling

Increase or decrease application instances.

Example:

```
2 Pods
   |
   v

5 Pods
```

---

## Self-healing

If a container crashes, Kubernetes automatically creates a replacement.

Example:

```
Desired:
3 Pods

Running:
2 Pods

Kubernetes:
Creates missing Pod
```

---

## Rolling Updates

Deploy new application versions gradually without stopping the entire application.

Example:

```
Version 1
Pod
Pod
Pod


Update


Version 2
Pod
Pod
Pod
```

---

## Service Discovery

Kubernetes provides stable networking between changing containers.

Pods can be destroyed and recreated, but Services provide a consistent way to access them.

---

# Kubernetes Architecture

A Kubernetes cluster consists of:

```
                 Kubernetes Cluster

              Control Plane
                    |
        -------------------------
        |                       |
     Node 1                  Node 2
        |                       |
      Pods                   Pods
```

---

# Cluster

A cluster is a group of machines that run Kubernetes workloads.

A cluster contains:

- Control Plane
- Worker Nodes

The control plane manages the cluster.

Worker nodes run applications.

---

# Control Plane

The control plane is responsible for maintaining the desired state of the cluster.

Main components:

## API Server

The main communication point for Kubernetes.

Commands like:

```bash
kubectl apply -f app.yaml
```

send requests to the API server.

---

## Scheduler

Decides where new Pods should run.

Example:

```
New Pod created

Scheduler checks nodes:

Node 1: full
Node 2: available

Place Pod on Node 2
```

---

## Controller Manager

Continuously checks the cluster state and makes corrections.

Example:

Desired:

```
3 replicas
```

Current:

```
2 replicas
```

Controller creates another Pod.

---

## etcd

A database storing Kubernetes cluster state.

Stores information such as:

- Applications
- Nodes
- Configuration
- Secrets
- Cluster metadata

---

# Node

A Node is a machine that runs applications.

Nodes can be:

- Physical machines
- Virtual machines
- Cloud instances

Each node runs:

## kubelet

An agent running on every node.

Responsible for:

- Starting Pods
- Reporting status
- Communicating with the control plane

---

## Container Runtime

The software that actually runs containers.

Examples:

- containerd
- CRI-O

---

## kube-proxy

Handles network communication between Pods and Services.

---

# Core Kubernetes Concepts

---

# Pod

A Pod is the smallest deployable unit in Kubernetes.

A Pod contains one or more containers.

Most Pods contain one container:

```
Pod
 |
 ÀÄÄ Container
```

Multiple containers can share:

- Network
- Storage
- Lifecycle

Example:

```
Pod

 ÃÄÄ Application container
 |
 ÀÄÄ Logging sidecar container
```

Pods are temporary.

If a Pod dies, Kubernetes usually creates a replacement.

---

# Deployment

A Deployment manages Pods.

Instead of manually creating Pods, you normally create a Deployment.

A Deployment defines:

- Which image to run
- How many replicas are needed
- Update strategy

Example:

```
Deployment

    |
    v

ReplicaSet

    |
    v

Pods
Pods
Pods
```

A Deployment provides:

- Scaling
- Self-healing
- Rolling updates
- Rollbacks

---

# ReplicaSet

A ReplicaSet ensures the correct number of Pods are running.

Example:

Desired:

```
3 Pods
```

Current:

```
2 Pods
```

ReplicaSet creates:

```
+1 Pod
```

Usually you do not create ReplicaSets directly. Deployments manage them automatically.

---

# Service

Pods are temporary.

Their IP addresses can change when they are recreated.

A Service provides a stable network endpoint for accessing Pods.

Example:

```
User

 |

Service

 |

Pod
Pod
Pod
```

Services provide:

- Stable IP address
- Load balancing
- Service discovery

Types:

## ClusterIP

Default service type.

Accessible only inside the cluster.

---

## NodePort

Exposes a service on a port on each node.

Used mainly for testing.

---

## LoadBalancer

Creates an external load balancer.

Common in cloud environments.

---

# Namespace

Namespaces organize resources inside a cluster.

Example:

```
Cluster

ÃÄÄ development
³     ÀÄÄ app
³
ÃÄÄ testing
³     ÀÄÄ app
³
ÀÄÄ production
      ÀÄÄ app
```

Useful for:

- Multiple teams
- Multiple environments
- Resource isolation

---

# ConfigMap

Stores non-sensitive configuration separately from application code.

Examples:

- Environment variables
- Configuration files
- Application settings

Example:

```
Application Image

+

ConfigMap

=

Running Application
```

---

# Secret

Stores sensitive information.

Examples:

- Passwords
- API keys
- Certificates
- Tokens

Secrets should not be committed directly into source control.

---

# Volume

Containers are temporary.

Data inside a container disappears when the container is removed.

Volumes provide persistent storage.

Example:

```
Pod

Application Container

        |
        |
      Volume

        |
        |
Persistent Storage
```

Used for:

- Databases
- Uploaded files
- Persistent application data

---

# PersistentVolume (PV)

A PersistentVolume represents storage available to the cluster.

Examples:

- Cloud disks
- Network storage
- Local storage

---

# PersistentVolumeClaim (PVC)

A request for storage by an application.

Example:

```
Application

requests:

20GB storage


PVC

gets storage from:


PV
```

---

# Ingress

Ingress manages external HTTP/HTTPS traffic into the cluster.

Example:

```
Internet

   |
   v

Ingress

   |
   +---- frontend service
   |
   +---- backend service
```

Ingress can route traffic based on:

- Domain name
- URL path

Example:

```
example.com/api

        |
        v

Backend Service


example.com/

        |
        v

Frontend Service
```

---

# Kubernetes Object Relationship

The common application flow:

```
Deployment

    |
    v

ReplicaSet

    |
    v

Pods

    |
    v

Containers
```

Networking:

```
User

 |
 v

Ingress

 |
 v

Service

 |
 v

Pods
```

Configuration:

```
Pod

 |
 +---- ConfigMap
 |
 +---- Secret
 |
 +---- Volume
```

---

# Docker vs Kubernetes

## Docker

Focus:

> How do I package and run my application?

Docker provides:

- Images
- Containers
- Networks
- Volumes

---

## Kubernetes

Focus:

> How do I run many containers reliably?

Kubernetes provides:

- Scheduling
- Scaling
- Networking
- Recovery
- Deployment management

---

# Typical Deployment Flow

```
Developer

    |
    v

Dockerfile

    |
    v

Docker Image

    |
    v

Container Registry

    |
    v

Kubernetes Deployment

    |
    v

Pods

    |
    v

Running Application
```
