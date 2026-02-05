import { api, type Stack } from "$lib/api";
import { activeTeam } from "$lib/stores";
import { toast } from "svelte-sonner";
import { get } from "svelte/store";

export class StackListController {
  stacks = $state<Stack[]>([]);
  loading = $state(true);
  creating = $state(false);
  actionLoading = $state<string | null>(null);

  // Dialog state
  showCreateDialog = $state(false);
  newStack = $state({ name: "", composeContent: "" });
  importMode = $state<"manual" | "git">("manual");
  gitStack = $state({
    url: "",
    branch: "main",
    composePath: "docker-compose.yml",
    envContent: "",
  });

  // UI States
  showRemoveConfirm = $state(false);
  stackToRemove = $state<string | null>(null);

  async init() {
    await this.loadStacks();
  }

  async loadStacks() {
    const team = get(activeTeam)?.team;
    if (!team) {
      this.stacks = [];
      this.loading = false;
      return;
    }
    this.loading = true;
    const result = await api.stacks.list(team.id);
    if (result.data) {
      this.stacks = result.data;
    }
    this.loading = false;
  }

  async createStack() {
    const team = get(activeTeam)?.team;
    if (!this.newStack.name || !team) return;
    this.creating = true;

    // Toast early
    const stackName = this.newStack.name;
    toast.info(
      `Deploying stack ${stackName}... This might take a while if images need pulling.`,
    );

    // Close dialog immediately as requested
    this.showCreateDialog = false;

    let result;
    try {
      if (this.importMode === "manual") {
        if (!this.newStack.composeContent) {
          this.creating = false;
          return;
        }
        result = await api.stacks.create({
          name: this.newStack.name,
          team_id: team.id,
          compose_content: this.newStack.composeContent,
        });
      } else {
        if (!this.gitStack.url) {
          this.creating = false;
          return;
        }
        // Parse .env content into key-value pairs
        const envVars: Record<string, string> = {};
        if (this.gitStack.envContent.trim()) {
          for (const line of this.gitStack.envContent.split("\n")) {
            const trimmed = line.trim();
            if (trimmed && !trimmed.startsWith("#")) {
              const eqIdx = trimmed.indexOf("=");
              if (eqIdx > 0) {
                const key = trimmed.substring(0, eqIdx).trim();
                const value = trimmed.substring(eqIdx + 1).trim();
                envVars[key] = value;
              }
            }
          }
        }
        result = await api.stacks.createFromGit({
          name: this.newStack.name,
          team_id: team.id,
          git_url: this.gitStack.url,
          git_branch: this.gitStack.branch,
          compose_path: this.gitStack.composePath,
          env_vars: Object.keys(envVars).length > 0 ? envVars : undefined,
        });
      }

      if (result.data) {
        toast.success(`Stack ${stackName} created successfully`);
        this.newStack = { name: "", composeContent: "" };
        this.gitStack = {
          url: "",
          branch: "main",
          composePath: "docker-compose.yml",
          envContent: "",
        };
        await this.loadStacks();
      } else {
        toast.error(
          result.message ||
            result.error ||
            `Failed to create stack ${stackName}`,
        );
      }
    } catch (err) {
      toast.error(`Network error while creating stack ${stackName}`);
    } finally {
      this.creating = false;
    }
  }

  async startStack(id: string) {
    this.actionLoading = id;
    const result = await api.stacks.start(id);
    if (!result.error) toast.success("Stack started");
    else toast.error(result.message || result.error);
    await this.loadStacks();
    this.actionLoading = null;
  }

  async stopStack(id: string) {
    this.actionLoading = id;
    const result = await api.stacks.stop(id);
    if (!result.error) toast.success("Stack stopped");
    else toast.error(result.message || result.error);
    await this.loadStacks();
    this.actionLoading = null;
  }

  requestRemove(id: string) {
    this.stackToRemove = id;
    this.showRemoveConfirm = true;
  }

  async confirmRemove() {
    if (!this.stackToRemove) return;
    const id = this.stackToRemove;
    this.showRemoveConfirm = false;
    this.actionLoading = id;

    const result = await api.stacks.remove(id);
    if (!result.error) toast.success("Stack removed");
    else toast.error(result.message || result.error);
    await this.loadStacks();
    this.actionLoading = null;
  }

  async restoreBackup(backup: any) {
    const team = get(activeTeam)?.team;
    if (!team) return;
    this.creating = true;
    const result = await api.stacks.restore(team.id, backup);
    if (result.data) {
      toast.success("Stack restored successfully");
      await this.loadStacks();
    } else {
      toast.error(result.message || result.error || "Failed to restore stack");
    }
    this.creating = false;
  }
}
