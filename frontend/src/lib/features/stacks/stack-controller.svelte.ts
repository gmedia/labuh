import {
  api,
  type Stack,
  type Container,
  type Domain,
  type DeploymentLog,
  type StackHealth,
  type EnvVar,
  type ContainerResource,
  type ResourceMetric,
} from "$lib/api";
import { toast } from "svelte-sonner";
import { goto } from "$app/navigation";
import { browser } from "$app/environment";
import { API_URL } from "$lib/api";

export class StackController {
  id: string;
  stack = $state<Stack | null>(null);
  containers = $state<Container[]>([]);
  domains = $state<Domain[]>([]);
  deployments = $state<DeploymentLog[]>([]);
  logs = $state<Map<string, string[]>>(new Map());
  health = $state<StackHealth | null>(null);
  envVars = $state<EnvVar[]>([]);
  resourceLimits = $state<ContainerResource[]>([]);
  metrics = $state<ResourceMetric[]>([]);

  loading = $state(true);
  actionLoading = $state(false);
  savingCompose = $state(false);
  savingAutomation = $state(false);
  savingResources = $state<Set<string>>(new Set());

  // UI States
  showComposeEditor = $state(false);
  editedCompose = $state("");
  selectedRange = $state("1h");
  selectedContainerLogs = $state<string | null>(null);
  showSecrets = $state<Set<string>>(new Set());
  showBuildLogs = $state(false);

  // Confimation Modals
  showRedeployConfirm = $state(false);
  redeployService = $state<string | undefined>(undefined);
  showRemoveStackConfirm = $state(false);
  showRollbackConfirm = $state(false);
  showRemoveDomainConfirm = $state(false);
  domainToRemove = $state<string | null>(null);
  showDeleteEnvConfirm = $state(false);
  envVarToDelete = $state<{ key: string; container: string } | null>(null);
  showRegenerateWebhookConfirm = $state(false);
  showScaleConfirm = $state(false);
  scaleServiceTarget = $state<string | null>(null);
  scaleReplicas = $state(1);
  isCreating = $state(false); // For async stack creation UX

  constructor(id: string) {
    this.id = id;
  }

  async init() {
    this.loading = true;
    await this.loadAll();
    this.loading = false;
  }

  async loadAll() {
    await this.loadStack();
    if (this.stack) {
      await Promise.all([
        this.loadContainers(),
        this.loadDomains(),
        this.loadDeployments(),
        this.loadHealth(),
        this.loadEnvVars(),
        this.loadResourceLimits(),
        this.loadMetrics(),
      ]);
    }
  }

  async loadStack() {
    const result = await api.stacks.get(this.id);
    if (result.data) {
      this.stack = result.data;
      this.editedCompose = this.stack.compose_content || "";
    }
  }

  async loadContainers() {
    const result = await api.stacks.containers(this.id);
    if (result.data) {
      this.containers = result.data;
    }
  }

  async loadDomains() {
    const result = await api.stacks.domains.list(this.id);
    if (result.data) {
      this.domains = result.data;
    }
  }

  async loadDeployments() {
    const result = await api.stacks.deploymentLogs(this.id);
    if (result.data) {
      this.deployments = result.data;
    }
  }

  async loadHealth() {
    const result = await api.stacks.health(this.id);
    if (result.data) {
      this.health = result.data;
    }
  }

  async loadEnvVars() {
    const result = await api.stacks.env.list(this.id);
    if (result.data) {
      this.envVars = result.data;
    }
  }

  async loadResourceLimits() {
    const result = await api.stacks.resources.getLimits(this.id);
    if (result.data) {
      let limits = result.data;
      const serviceNames = this.containers.map(
        (c) =>
          c.labels?.["labuh.service.name"] || c.names[0]?.replace(/^\//, ""),
      );
      for (const name of serviceNames) {
        if (!limits.find((l) => l.service_name === name)) {
          limits.push({
            id: "",
            stack_id: this.id,
            service_name: name,
            cpu_limit: undefined,
            memory_limit: undefined,
            created_at: "",
            updated_at: "",
          });
        }
      }
      this.resourceLimits = limits;
    }
  }

  async loadMetrics() {
    const result = await api.stacks.resources.getMetrics(
      this.id,
      this.selectedRange,
    );
    if (result.data) {
      this.metrics = result.data;
    }
  }

  async updateResourceLimit(
    serviceName: string,
    cpuLimit: number | undefined,
    memoryLimit: number | undefined,
  ) {
    this.savingResources.add(serviceName);
    const memBytes = memoryLimit ? memoryLimit * 1024 * 1024 : undefined;
    const result = await api.stacks.resources.updateLimits(
      this.id,
      serviceName,
      {
        cpu_limit: cpuLimit,
        memory_limit: memBytes,
      },
    );

    if (result.error) {
      toast.error(`Error: ${result.error}`);
    } else {
      toast.success(`Limits updated for ${serviceName}. Redeploy to apply.`);
      await this.loadResourceLimits();
    }
    this.savingResources.delete(serviceName);
  }

  async loadContainerLogs(containerId: string) {
    this.selectedContainerLogs = containerId;
    const result = await api.containers.logs(containerId, 100);
    if (result.data) {
      this.logs.set(containerId, result.data);
      this.logs = new Map(this.logs);
    }
  }

  async start() {
    this.actionLoading = true;
    await api.stacks.start(this.id);
    await Promise.all([this.loadStack(), this.loadContainers()]);
    this.actionLoading = false;
  }

  async stop() {
    this.actionLoading = true;
    await api.stacks.stop(this.id);
    await Promise.all([this.loadStack(), this.loadContainers()]);
    this.actionLoading = false;
  }

  requestRedeploy(serviceName?: string) {
    this.redeployService = serviceName;
    this.showRedeployConfirm = true;
  }

  async confirmRedeploy() {
    const serviceName = this.redeployService;
    this.showRedeployConfirm = false;
    this.actionLoading = true;
    try {
      await api.stacks.redeploy(this.id, serviceName);
      await Promise.all([
        this.loadStack(),
        this.loadContainers(),
        this.loadHealth(),
      ]);
    } catch (err: any) {
      toast.error(err.message || "Redeployment failed");
    } finally {
      this.actionLoading = false;
      this.redeployService = undefined;
    }
  }

  async build(serviceName?: string) {
    this.actionLoading = true;
    this.showBuildLogs = true;
    const result = await api.stacks.build(this.id, serviceName);
    if (result.error) {
      toast.error(result.message || result.error);
    } else {
      toast.success("Build triggered");
    }
    this.actionLoading = false;
  }

  requestRemove() {
    this.showRemoveStackConfirm = true;
  }

  async confirmRemove() {
    this.showRemoveStackConfirm = false;
    this.actionLoading = true;
    try {
      const res = await api.stacks.remove(this.id);
      if (res.error) {
        toast.error(res.error);
      } else {
        toast.success("Stack removed");
        goto("/dashboard/stacks");
      }
    } catch (err: any) {
      toast.error(err.message || "Failed to remove stack");
    } finally {
      this.actionLoading = false;
    }
  }

  requestRollback() {
    this.showRollbackConfirm = true;
  }

  async confirmRollback() {
    this.showRollbackConfirm = false;
    this.actionLoading = true;
    try {
      const result = await api.stacks.rollback(this.id);
      if (result.error) {
        toast.error(result.message || result.error);
      } else {
        toast.success("Rollback triggered");
        await Promise.all([
          this.loadStack(),
          this.loadContainers(),
          this.loadHealth(),
        ]);
      }
    } catch (e: any) {
      toast.error(e.message || "Failed to rollback stack");
    } finally {
      this.actionLoading = false;
    }
  }

  async saveCompose() {
    this.savingCompose = true;
    try {
      const result = await api.stacks.updateCompose(
        this.id,
        this.editedCompose,
      );
      if (result.error) {
        toast.error(result.message || result.error);
      } else {
        toast.success("Stack updated and redeployment triggered");
        this.showComposeEditor = false;
        await Promise.all([
          this.loadStack(),
          this.loadContainers(),
          this.loadHealth(),
        ]);
      }
    } catch (e: any) {
      toast.error(e.message || "Failed to update stack");
    } finally {
      this.savingCompose = false;
    }
  }

  async export() {
    this.actionLoading = true;
    try {
      const result = await api.stacks.backup(this.id);
      if (result.data) {
        const blob = new Blob([JSON.stringify(result.data, null, 2)], {
          type: "application/json",
        });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `labuh-backup-${this.stack?.name}-${new Date().toISOString().split("T")[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        toast.success("Backup downloaded");
      }
    } catch (e: any) {
      toast.error(e.message || "Failed to export stack");
    } finally {
      this.actionLoading = false;
    }
  }

  async syncGit() {
    this.actionLoading = true;
    this.showBuildLogs = true;
    try {
      const result = await api.stacks.syncGit(this.id);
      if (result.data) {
        toast.success("Stack synced with Git and redeployed");
        await Promise.all([
          this.loadStack(),
          this.loadContainers(),
          this.loadHealth(),
        ]);
      } else {
        toast.error(result.message || result.error || "Failed to sync Git");
      }
    } catch (e: any) {
      toast.error(e.message || "Failed to sync Git");
    } finally {
      this.actionLoading = false;
    }
  }

  requestRemoveDomain(domain: string) {
    this.domainToRemove = domain;
    this.showRemoveDomainConfirm = true;
  }

  async confirmRemoveDomain() {
    if (!this.domainToRemove) return;
    const domain = this.domainToRemove;
    this.showRemoveDomainConfirm = false;
    await api.stacks.domains.remove(this.id, domain);
    await this.loadDomains();
    this.domainToRemove = null;
  }

  async verifyDomain(domain: string) {
    await api.stacks.domains.verify(this.id, domain);
    await this.loadDomains();
  }

  async toggleBranding(domain: string, showBranding: boolean) {
    const result = await api.stacks.domains.toggleBranding(
      this.id,
      domain,
      showBranding,
    );
    if (result.error) {
      toast.error(result.message || result.error);
    } else {
      toast.success(
        `Labuh badge ${showBranding ? "enabled" : "disabled"} for ${domain}`,
      );
      await this.loadDomains();
    }
  }

  async saveAutomation(payload: {
    cron_schedule: string;
    health_check_path: string;
    health_check_interval: number;
  }) {
    this.savingAutomation = true;
    try {
      const result = await api.stacks.updateAutomation(this.id, payload);
      if (result.error) {
        toast.error(result.message || result.error);
      } else {
        toast.success("Automation settings updated");
        await this.loadStack();
      }
    } catch (e: any) {
      toast.error(e.message || "Failed to update automation settings");
    } finally {
      this.savingAutomation = false;
    }
  }

  async toggleSecretVisibility(envId: string) {
    const newSet = new Set(this.showSecrets);
    if (newSet.has(envId)) {
      newSet.delete(envId);
    } else {
      newSet.add(envId);
    }
    this.showSecrets = newSet;
  }

  async addEnvVar(payload: {
    container_name: string;
    key: string;
    value: string;
    is_secret: boolean;
  }) {
    await api.stacks.env.set(this.id, payload);
    await this.loadEnvVars();
  }

  requestDeleteEnvVar(key: string, containerName: string) {
    this.envVarToDelete = { key, container: containerName };
    this.showDeleteEnvConfirm = true;
  }

  async confirmDeleteEnvVar() {
    if (!this.envVarToDelete) return;
    const { key, container } = this.envVarToDelete;
    this.showDeleteEnvConfirm = false;
    await api.stacks.env.delete(this.id, key, container);
    await this.loadEnvVars();
    this.envVarToDelete = null;
  }

  requestRegenerateWebhook() {
    this.showRegenerateWebhookConfirm = true;
  }

  async confirmRegenerateWebhook() {
    this.showRegenerateWebhookConfirm = false;
    const result = await api.stacks.regenerateWebhookToken(this.id);
    if (result.data && this.stack) {
      this.stack.webhook_token = result.data.token;
    }
  }

  requestScale(serviceName: string) {
    this.scaleServiceTarget = serviceName;
    // Find current replicas if possible, or default to 1
    const parts = serviceName.split("_");
    const container = this.containers.find(
      (c) =>
        c.labels?.["com.docker.swarm.service.name"] ===
        `${this.stack?.name}_${serviceName}`,
    );
    this.scaleReplicas = 1; // Default
    this.showScaleConfirm = true;
  }

  async confirmScale(replicas: number) {
    if (!this.scaleServiceTarget) return;
    this.showScaleConfirm = false;
    this.actionLoading = true;
    try {
      const result = await api.stacks.scale(
        this.id,
        this.scaleServiceTarget,
        replicas,
      );
      if (result.error) {
        toast.error(result.message || result.error);
      } else {
        toast.success(`Scaling ${this.scaleServiceTarget} to ${replicas}...`);
        // Swarm updates are async, so we'll just reload containers after a short delay
        setTimeout(() => this.loadContainers(), 2000);
      }
    } catch (e: any) {
      toast.error(e.message || "Failed to scale service");
    } finally {
      this.actionLoading = false;
      this.scaleServiceTarget = null;
    }
  }

  get runningCount() {
    return this.containers.filter((c) => c.state === "running").length;
  }

  get stoppedCount() {
    return this.containers.filter((c) => c.state !== "running").length;
  }

  get webhookUrl() {
    if (!this.stack?.webhook_token) return "";
    const base = browser ? window.location.origin : API_URL;
    return `${base}/api/webhooks/deploy/${this.id}/${this.stack.webhook_token}`;
  }
}
