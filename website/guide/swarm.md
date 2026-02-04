# Swarm & Clusters

Labuh v0.4.0 menghadirkan dukungan native untuk **Docker Swarm**, memungkinkan Anda mengelola cluster dari banyak server (node) langsung dari satu dashboard.

## Swarm Overview

Docker Swarm turns a pool of Docker hosts into a single virtual server. Labuh integrates seamlessly with Swarm to provide:

- **Multi-Node Deployment**: Deploy stacks across multiple servers.
- **High Availability**: Services can be replicated across nodes.
- **Unified Management**: Manage Managers and Workers from one UI.

## Initializing Swarm

If your server isn't part of a Swarm yet, you can initialize it directly from Labuh:

1. Go to **Nodes** sidebar menu.
2. Click **Initialize Swarm**.
3. Labuh will configure the current server as the first **Manager** node.

## Node Management

Once Swarm is active, the **Nodes** page becomes your cluster command center.

### Joining Nodes

To add more servers to your cluster:

1. Click **Join Token** on the Manager node.
2. Copy the **Worker Token** (or Manager Token).
3. On the new server (which must have Docker installed), run the provided `docker swarm join` command OR use the Labuh interface if Labuh is installed there too.

### Promoting & Demoting

- **Promote to Manager**: Gives a Worker node administrative powers.
- **Demote to Worker**: Strips administrative powers (useful for maintenance).

You can perform these actions by clicking the **...** (Options) button on any node card.

### Terminal & Monitoring

- **Node Terminal**: Akses shell host server langsung dari browser menggunakan ikon terminal di setiap kartu node. Sangat berguna untuk maintenance server jarak jauh.
- **Resource Monitoring**: Pantau kapasitas CPU, RAM, dan Disk dari setiap node secara real-time.

## Network Visualization

Labuh menyediakan **Network Visualizer** yang powerful untuk membantu Anda memahami topologi jaringan di dalam cluster.

- **Interactive Graph**: Lihat bagaimana container terhubung ke network dan container lainnya.
- **Real-time Status**: Node berwarna menunjukkan kesehatan service Anda.

Akses visualizer melalui menu **Networks** di sidebar dashboard.
