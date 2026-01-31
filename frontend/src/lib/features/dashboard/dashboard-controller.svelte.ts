import {
  api,
  type SystemStats,
  type Container,
  type Image,
  type Stack,
} from "$lib/api";
import { activeTeam } from "$lib/stores";
import { get } from "svelte/store";

export class DashboardController {
  systemHealth = $state<{ status: string; version: string } | null>(null);
  systemStats = $state<SystemStats | null>(null);
  containers = $state<Container[]>([]);
  images = $state<Image[]>([]);
  stacks = $state<Stack[]>([]);
  loading = $state(true);

  async init() {
    await this.loadAll();
  }

  async loadAll() {
    const team = get(activeTeam)?.team;
    if (!team) {
      this.containers = [];
      this.stacks = [];
      this.loading = false;
      const [healthRes, statsRes] = await Promise.all([
        api.health.check(),
        api.system.stats(),
      ]);
      if (healthRes.data) this.systemHealth = healthRes.data;
      if (statsRes.data) this.systemStats = statsRes.data;
      return;
    }

    this.loading = true;
    const [healthRes, statsRes, containersRes, imagesRes, stacksRes] =
      await Promise.all([
        api.health.check(),
        api.system.stats(),
        api.containers.list(true, team.id),
        api.images.list(), // Images are host-wide
        api.stacks.list(team.id),
      ]);

    if (healthRes.data) this.systemHealth = healthRes.data;
    if (statsRes.data) this.systemStats = statsRes.data;
    if (containersRes.data) this.containers = containersRes.data;
    if (imagesRes.data) this.images = imagesRes.data;
    if (stacksRes.data) this.stacks = stacksRes.data;

    this.loading = false;
  }

  get runningContainers() {
    return this.containers.filter((c) => c.state === "running").length;
  }

  get runningStacks() {
    return this.stacks.filter((s) => s.status === "running").length;
  }
}
