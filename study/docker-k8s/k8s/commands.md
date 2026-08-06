# Kubernetes Commands

This document covers common `kubectl` commands used for deploying, inspecting, debugging, and managing Kubernetes applications.

---

# 1. Cluster Information

## kubectl cluster-info

### What it does

Displays information about the Kubernetes control plane and cluster services.

### Example

```bash
kubectl cluster-info
```

Useful when:

- Checking if your cluster is running.
- Verifying your kubectl connection.

---

## kubectl get nodes

### What it does

Lists all nodes in the cluster.

### Example

```bash
kubectl get nodes
```

Example output:

```
NAME        STATUS   ROLES
node-1      Ready    worker
node-2      Ready    worker
```

Useful for:

- Checking cluster health.
- Seeing available machines.

---

## kubectl version

### What it does

Shows kubectl and Kubernetes API server versions.

### Example

```bash
kubectl version
```

Useful when:

- Debugging version compatibility issues.

---

# 2. Working With Resources

## kubectl get

### What it does

Lists Kubernetes resources.

### Syntax

```bash
kubectl get RESOURCE
```

Examples:

List Pods:

```bash
kubectl get pods
```

List Deployments:

```bash
kubectl get deployments
```

List Services:

```bash
kubectl get services
```

---

Show resources in all namespaces:

```bash
kubectl get pods -A
```

---

Get detailed information:

```bash
kubectl get pods -o wide
```

Shows:

- Pod IP
- Node location
- Additional details

---

# 3. Creating and Updating Resources

## kubectl apply

### What it does

Creates or updates Kubernetes resources from YAML files.

### Example

```bash
kubectl apply -f deployment.yaml
```

Kubernetes compares the YAML with the current state and makes changes needed to reach the desired state.

Commonly used for:

- Deployments
- Services
- ConfigMaps
- Secrets

---

## kubectl delete

### What it does

Deletes Kubernetes resources.

### Delete using YAML:

```bash
kubectl delete -f deployment.yaml
```

Delete by name:

```bash
kubectl delete pod my-pod
```

---

# 4. Inspecting Applications

## kubectl describe

### What it does

Shows detailed information about a Kubernetes resource.

### Example:

```bash
kubectl describe pod my-pod
```

Shows:

- Events
- Container status
- Network information
- Volumes
- Errors

Very useful for troubleshooting.

---

## kubectl logs

### What it does

Displays logs from a container inside a Pod.

### Example:

```bash
kubectl logs my-pod
```

Follow logs:

```bash
kubectl logs -f my-pod
```

Useful for:

- Application errors
- Startup problems
- Debugging

---

## kubectl exec

### What it does

Runs a command inside a container.

### Example:

```bash
kubectl exec -it my-pod -- bash
```

Options:

- `-i` keeps input open.
- `-t` creates an interactive terminal.

Useful for:

- Inspecting files.
- Checking environment variables.
- Debugging running containers.

---

# 5. Deployments

## kubectl rollout status

### What it does

Shows the progress of a Deployment update.

### Example:

```bash
kubectl rollout status deployment/my-app
```

Useful when:

- Deploying a new version.
- Checking if Pods became healthy.

---

## kubectl rollout history

### What it does

Shows previous Deployment versions.

### Example:

```bash
kubectl rollout history deployment/my-app
```

---

## kubectl rollout undo

### What it does

Rolls back a Deployment to a previous version.

### Example:

```bash
kubectl rollout undo deployment/my-app
```

Useful when:

- A new release breaks the application.

---

## kubectl scale

### What it does

Changes the number of Pod replicas.

### Example:

```bash
kubectl scale deployment my-app --replicas=5
```

Before:

```
3 Pods
```

After:

```
5 Pods
```

---

# 6. Namespaces

## kubectl get namespaces

### What it does

Lists namespaces.

### Example:

```bash
kubectl get namespaces
```

---

## kubectl create namespace

### What it does

Creates a new namespace.

### Example:

```bash
kubectl create namespace development
```

---

## Using a namespace

Specify namespace:

```bash
kubectl get pods -n development
```

Set default namespace:

```bash
kubectl config set-context --current --namespace=development
```

---

# 7. Services and Networking

## kubectl get services

### What it does

Lists Kubernetes Services.

### Example:

```bash
kubectl get services
```

Shows:

- Service name
- Type
- Cluster IP
- Ports

---

## kubectl port-forward

### What it does

Creates a temporary connection from your machine to a Pod.

### Example:

```bash
kubectl port-forward pod/my-pod 8080:80
```

Maps:

```
localhost:8080

      |

Pod port 80
```

Useful for:

- Local testing.
- Debugging without exposing a service.

---

# 8. Configuration

## kubectl config get-contexts

### What it does

Lists available Kubernetes clusters.

### Example:

```bash
kubectl config get-contexts
```

---

## kubectl config use-context

### What it does

Switches between Kubernetes clusters.

### Example:

```bash
kubectl config use-context my-cluster
```

Useful when working with:

- Local clusters.
- Test environments.
- Production clusters.

---

# 9. Debugging

## kubectl get events

### What it does

Shows recent cluster events.

### Example:

```bash
kubectl get events
```

Useful for finding:

- Failed scheduling
- Image pull errors
- Container crashes

---

## kubectl top

### What it does

Shows resource usage.

### Example:

```bash
kubectl top pods
```

Shows:

- CPU usage
- Memory usage

Useful for:

- Performance debugging.
- Finding resource problems.

---

# Common Workflow

Deploy an application:

```bash
kubectl apply -f deployment.yaml
```

Check Pods:

```bash
kubectl get pods
```

Inspect problems:

```bash
kubectl describe pod my-pod
```

View logs:

```bash
kubectl logs my-pod
```

Enter container:

```bash
kubectl exec -it my-pod -- bash
```

Update application:

```bash
kubectl apply -f deployment.yaml
```

Check rollout:

```bash
kubectl rollout status deployment/my-app
```

Rollback if needed:

```bash
kubectl rollout undo deployment/my-app
```
