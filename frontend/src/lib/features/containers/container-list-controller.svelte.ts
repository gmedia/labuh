import { api, type Container } from "$lib/api";
import { activeTeam } from "$lib/stores";
import { toast } from "svelte-sonner";
import { get } from "svelte/store";

export class ContainerListController {
  containers = $state<Container[]>([]);
  loading = $state(true);
  showCreateDialog = $state(false);
  newContainer = $state({
    name: "",
    image: "",
    envVars: [] as { key: string; value: string; masked: boolean }[],
    ports: [] as { hostPort: string; containerPort: string }[],
  });
  creating = $state(false);
  actionLoading = $state<string | null>(null);
  envImportText = $state("");
  showEnvImport = $state(false);

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
    }
    this.loading = false;
  }

  addEnvVar() {
    this.newContainer.envVars = [
      ...this.newContainer.envVars,
      { key: "", value: "", masked: false },
    ];
  }

  removeEnvVar(index: number) {
    this.newContainer.envVars = this.newContainer.envVars.filter(
      (_, i) => i !== index,
    );
  }

  addPort() {
    this.newContainer.ports = [
      ...this.newContainer.ports,
      { hostPort: "", containerPort: "" },
    ];
  }

  removePort(index: number) {
    this.newContainer.ports = this.newContainer.ports.filter(
      (_, i) => i !== index,
    );
  }

  importEnvFromText() {
    const lines = this.envImportText
      .split("\n")
      .filter((line) => line.trim() && !line.startsWith("#"));
    const newVars = lines
      .map((line) => {
        const [key, ...valueParts] = line.split("=");
        return {
          key: key?.trim() || "",
          value: valueParts
            .join("=")
            .trim()
            .replace(/^["']|["']$/g, ""),
          masked:
            key?.toLowerCase().includes("secret") ||
            key?.toLowerCase().includes("password") ||
            key?.toLowerCase().includes("key"),
        };
      })
      .filter((v) => v.key);

    this.newContainer.envVars = [...this.newContainer.envVars, ...newVars];
    this.envImportText = "";
    this.showEnvImport = false;
  }

  async loadImagePorts() {
    if (!this.newContainer.image) return;
    const result = await api.images.inspect(this.newContainer.image);
    if (result.data?.exposed_ports) {
      const newPorts = result.data.exposed_ports.map((port) => ({
        hostPort: "",
        containerPort: port.replace("/tcp", "").replace("/udp", ""),
      }));
      this.newContainer.ports = [...this.newContainer.ports, ...newPorts];
    }
  }

  async createContainer() {
    if (!this.newContainer.name || !this.newContainer.image) return;
    this.creating = true;

    const env = this.newContainer.envVars
      .filter((e) => e.key)
      .map((e) => `${e.key}=${e.value}`);

    const ports: Record<string, string> = {};
    this.newContainer.ports
      .filter((p) => p.containerPort && p.hostPort)
      .forEach((p) => {
        ports[p.containerPort] = p.hostPort;
      });

    const result = await api.containers.create({
      name: this.newContainer.name,
      image: this.newContainer.image,
      env: env.length > 0 ? env : undefined,
      ports: Object.keys(ports).length > 0 ? ports : undefined,
    });

    if (result.data) {
      toast.success("Container created successfully");
      this.showCreateDialog = false;
      this.newContainer = { name: "", image: "", envVars: [], ports: [] };
      await this.loadContainers();
    } else {
      toast.error(
        result.message || result.error || "Failed to create container",
      );
    }
    this.creating = false;
  }

  async startContainer(id: string) {
    this.actionLoading = id;
    const result = await api.containers.start(id);
    if (!result.error) toast.success("Container started");
    else toast.error(result.message || result.error);
    await this.loadContainers();
    this.actionLoading = null;
  }

  async stopContainer(id: string) {
    this.actionLoading = id;
    const result = await api.containers.stop(id);
    if (!result.error) toast.success("Container stopped");
    else toast.error(result.message || result.error);
    await this.loadContainers();
    this.actionLoading = null;
  }

  async restartContainer(id: string) {
    this.actionLoading = id;
    const result = await api.containers.restart(id);
    if (!result.error) toast.success("Container restarted");
    else toast.error(result.message || result.error);
    await this.loadContainers();
    this.actionLoading = null;
  }

  async removeContainer(id: string) {
    if (!confirm("Are you sure you want to delete this container?")) return;
    this.actionLoading = id;
    const result = await api.containers.remove(id);
    if (!result.error) toast.success("Container removed");
    else toast.error(result.message || result.error);
    await this.loadContainers();
    this.actionLoading = null;
  }
}
