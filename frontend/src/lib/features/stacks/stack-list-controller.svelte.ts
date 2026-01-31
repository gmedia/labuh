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
  });

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

    let result;
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
      result = await api.stacks.createFromGit({
        name: this.newStack.name,
        team_id: team.id,
        git_url: this.gitStack.url,
        git_branch: this.gitStack.branch,
        compose_path: this.gitStack.composePath,
      });
    }

    if (result.data) {
      toast.success("Stack created successfully");
      this.showCreateDialog = false;
      this.newStack = { name: "", composeContent: "" };
      this.gitStack = {
        url: "",
        branch: "main",
        composePath: "docker-compose.yml",
      };
      await this.loadStacks();
    } else {
      toast.error(result.message || result.error || "Failed to create stack");
    }
    this.creating = false;
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

  async removeStack(id: string) {
    if (
      !confirm(
        "Are you sure you want to delete this stack and all its containers?",
      )
    )
      return;
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
