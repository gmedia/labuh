import { api, type Container } from "$lib/api";
import { activeTeam } from "$lib/stores";
import { get } from "svelte/store";

export class LogsController {
  containers = $state<Container[]>([]);
  selectedContainerId = $state<string | null>(null);
  logs = $state<string[]>([]);
  loading = $state(true);
  logsLoading = $state(false);
  searchQuery = $state("");

  async init() {
    await this.loadContainers();
  }

  async loadContainers() {
    const team = get(activeTeam)?.team;
    if (!team) {
      this.containers = [];
      this.loading = false;
      return;
    }
    this.loading = true;
    const result = await api.containers.list(true, team.id);
    if (result.data) {
      this.containers = result.data;
      if (this.containers.length > 0 && !this.selectedContainerId) {
        this.selectedContainerId = this.containers[0].id;
        await this.loadLogs();
      }
    }
    this.loading = false;
  }

  async selectContainer(id: string) {
    this.selectedContainerId = id;
    await this.loadLogs();
  }

  async loadLogs() {
    if (!this.selectedContainerId) return;
    this.logsLoading = true;
    const result = await api.containers.logs(this.selectedContainerId, 500);
    if (result.data) {
      this.logs = result.data;
    }
    this.logsLoading = false;
  }

  downloadLogs() {
    const content = this.logs.join("\n");
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    const name =
      this.selectedContainer?.names[0]?.replace(/^\//, "") ||
      this.selectedContainerId?.slice(0, 12);
    a.download = `logs-${name}-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  }

  get selectedContainer() {
    return this.containers.find((c) => c.id === this.selectedContainerId);
  }

  get filteredLogs() {
    if (!this.searchQuery) return this.logs;
    const q = this.searchQuery.toLowerCase();
    return this.logs.filter((line) => line.toLowerCase().includes(q));
  }
}
