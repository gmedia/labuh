import { api, type Container, type ContainerStats } from "$lib/api";
import { goto } from "$app/navigation";

export class ContainerController {
  id: string;
  container = $state<Container | null>(null);
  stats = $state<ContainerStats | null>(null);
  logs = $state<string[]>([]);
  loading = $state(true);
  logsLoading = $state(false);
  actionLoading = $state(false);

  constructor(id: string) {
    this.id = id;
  }

  async init() {
    await this.loadContainer();
    if (this.container) {
      await Promise.all([this.loadStats(), this.loadLogs()]);
    }
  }

  async loadContainer() {
    const result = await api.containers.list(true);
    if (result.data) {
      this.container =
        result.data.find((c) => c.id === this.id || c.id.startsWith(this.id)) ||
        null;
    }
    this.loading = false;
  }

  async loadStats() {
    if (!this.container) return;
    const result = await api.containers.stats(this.container.id);
    if (result.data) {
      this.stats = result.data;
    }
  }

  async loadLogs() {
    if (!this.container) return;
    this.logsLoading = true;
    const result = await api.containers.logs(this.container.id, 200);
    if (result.data) {
      this.logs = result.data;
    }
    this.logsLoading = false;
  }

  async start() {
    this.actionLoading = true;
    await api.containers.start(this.id);
    await this.loadContainer();
    this.actionLoading = false;
  }

  async stop() {
    this.actionLoading = true;
    await api.containers.stop(this.id);
    await this.loadContainer();
    this.actionLoading = false;
  }

  async restart() {
    this.actionLoading = true;
    await api.containers.restart(this.id);
    await this.loadContainer();
    this.actionLoading = false;
  }

  async remove() {
    if (!confirm("Are you sure you want to delete this container?")) return;
    this.actionLoading = true;
    await api.containers.remove(this.id);
    goto("/dashboard/containers");
  }
}
