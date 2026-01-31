import { api, type RegistryCredential } from "$lib/api";
import { auth, activeTeam } from "$lib/stores";
import { toast } from "svelte-sonner";
import { get } from "svelte/store";

export class SettingsController {
  name = $state(get(auth).user?.name || "");
  registries = $state<RegistryCredential[]>([]);
  loadingRegistries = $state(true);
  newRegistry = $state({
    name: "",
    registry_url: "",
    username: "",
    password: "",
  });
  addingRegistry = $state(false);

  async init() {
    await this.loadRegistries();
  }

  async loadRegistries() {
    const team = get(activeTeam)?.team;
    if (!team) {
      this.registries = [];
      this.loadingRegistries = false;
      return;
    }
    this.loadingRegistries = true;
    const result = await api.registries.list(team.id);
    if (result.data) {
      this.registries = result.data;
    }
    this.loadingRegistries = false;
  }

  async addRegistry() {
    const team = get(activeTeam)?.team;
    if (
      !this.newRegistry.name ||
      !this.newRegistry.registry_url ||
      !this.newRegistry.username ||
      !this.newRegistry.password ||
      !team
    ) {
      return;
    }
    this.addingRegistry = true;
    const result = await api.registries.add({
      ...this.newRegistry,
      team_id: team.id,
    });
    if (result.data) {
      toast.success("Registry credential added");
      this.registries = [result.data, ...this.registries];
      this.newRegistry = {
        name: "",
        registry_url: "",
        username: "",
        password: "",
      };
    } else {
      toast.error(result.message || result.error || "Failed to add registry");
    }
    this.addingRegistry = false;
  }

  async removeRegistry(id: string) {
    const team = get(activeTeam)?.team;
    if (!team) return;
    if (!confirm("Are you sure you want to remove this registry credential?"))
      return;
    const result = await api.registries.remove(id, team.id);
    if (!result.error) {
      toast.success("Registry credential removed");
      this.registries = this.registries.filter((r) => r.id !== id);
    } else {
      toast.error(result.message || result.error);
    }
  }

  async saveProfile() {
    // Backend doesn't have profile update API yet, skipping for now
    toast.info("Profile update not yet implemented on backend");
  }
}
