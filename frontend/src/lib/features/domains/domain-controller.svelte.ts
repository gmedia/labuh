import {
  api,
  type Domain,
  type DnsConfig,
  type RemoteDnsRecord,
} from "$lib/api";
import { activeTeam } from "$lib/stores";
import { toast } from "svelte-sonner";
import { get } from "svelte/store";

export class DomainController {
  domains = $state<Domain[]>([]);
  dnsConfigs = $state<DnsConfig[]>([]);
  remoteRecords = $state<RemoteDnsRecord[]>([]);
  loading = $state(false);
  loadingRemote = $state(false);

  // UI States
  showDnsDialog = $state(false);
  showRegisterDialog = $state(false);
  showDiscoveryView = $state(false);
  selectedProvider = $state("Cloudflare");
  dnsConfigFields = $state<any>({});

  // Registration State
  availableBaseDomains = $state<string[]>([]);
  stacks = $state<any[]>([]);
  containers = $state<any[]>([]);
  selectedStackId = $state("");
  selectedBaseDomain = $state("");
  subdomain = $state("");
  selectedContainer = $state("");
  selectedPort = $state(80);
  registrationLoading = $state(false);
  selectedRemoteRecord = $state<RemoteDnsRecord | null>(null);

  async init() {
    await this.loadAll();
  }

  async loadAll() {
    this.loading = true;
    try {
      const team = get(activeTeam);
      if (team?.team?.id) {
        const domainRes = await api.stacks.domains.listAll(team.team.id);
        if (domainRes.data) {
          this.domains = domainRes.data;
        }

        const dnsRes = await api.dns.listConfigs(team.team.id);
        if (dnsRes.data) {
          this.dnsConfigs = dnsRes.data;
        }
      }
    } catch (err) {
      console.error("Failed to load domains data", err);
    } finally {
      this.loading = false;
    }
  }

  async saveDnsConfig() {
    const team = get(activeTeam);
    if (!team?.team?.id) return;

    try {
      const res = await api.dns.saveConfig(
        team.team.id,
        this.selectedProvider,
        this.dnsConfigFields,
      );

      if (res.data) {
        toast.success(`${this.selectedProvider} configuration saved`);
        this.showDnsDialog = false;
        await this.loadAll();
      } else {
        toast.error(res.error || "Failed to save configuration");
      }
    } catch (err) {
      toast.error("Network error while saving configuration");
    }
  }

  async removeDnsConfig(provider: string) {
    const team = get(activeTeam);
    if (!team?.team?.id) return;
    if (!confirm(`Are you sure you want to remove ${provider} configuration?`))
      return;

    try {
      const res = await api.dns.deleteConfig(team.team.id, provider);
      if (!res.error) {
        toast.success(`${provider} configuration removed`);
        await this.loadAll();
      } else {
        toast.error(res.error);
      }
    } catch (err) {
      toast.error("Failed to remove configuration");
    }
  }

  async verifyDomain(stackId: string, domain: string) {
    try {
      const res = await api.stacks.domains.verify(stackId, domain);
      if (res.data?.verified) {
        toast.success(`Domain ${domain} verified!`);
        await this.loadAll();
      } else {
        toast.error(
          `Verification failed for ${domain}. Please check DNS records.`,
        );
      }
    } catch (err) {
      toast.error("Verification error");
    }
  }

  async removeDomain(stackId: string, domain: string) {
    if (!confirm(`Are you sure you want to remove ${domain}?`)) return;

    try {
      const res = await api.stacks.domains.remove(stackId, domain);
      if (!res.error) {
        toast.success(`Domain ${domain} removed`);
        await this.loadAll();
      } else {
        toast.error(res.error);
      }
    } catch (err) {
      toast.error("Failed to remove domain");
    }
  }

  async openRegisterDialog() {
    const team = get(activeTeam);
    if (!team?.team?.id) return;

    this.showRegisterDialog = true;
    this.loading = true;

    try {
      // Fetch stacks for the team
      const stacksRes = await api.stacks.list(team.team.id);
      if (stacksRes.data) {
        this.stacks = stacksRes.data;
      }

      // If we have a provider configured, fetch available domains
      if (this.dnsConfigs.length > 0) {
        await this.fetchAvailableDomains(this.dnsConfigs[0].provider);
      }
    } finally {
      this.loading = false;
    }
  }

  async fetchAvailableDomains(provider: string) {
    const team = get(activeTeam);
    if (!team?.team?.id) return;

    try {
      const res = await api.dns.listAvailableDomains(team.team.id, provider);
      if (res.data) {
        this.availableBaseDomains = res.data;
        if (res.data.length > 0) {
          this.selectedBaseDomain = res.data[0];
        }
      }
    } catch (err) {
      console.error("Failed to fetch available domains", err);
    }
  }

  async fetchContainers(stackId: string) {
    if (!stackId) return;
    try {
      const res = await api.stacks.containers(stackId);
      if (res.data) {
        this.containers = res.data;
        if (res.data.length > 0) {
          this.selectedContainer = res.data[0].names[0].replace("/", "");
        }
      }
    } catch (err) {
      console.error("Failed to fetch containers", err);
    }
  }

  async registerDomain() {
    if (!this.selectedStackId || !this.selectedContainer) {
      toast.error("Please select a stack and container");
      return;
    }

    const fullDomain = this.subdomain
      ? `${this.subdomain}.${this.selectedBaseDomain}`
      : this.selectedBaseDomain;

    this.registrationLoading = true;
    try {
      const res = await api.stacks.domains.add(this.selectedStackId, {
        domain: fullDomain,
        container_name: this.selectedContainer,
        container_port: this.selectedPort,
        provider: this.selectedProvider as any,
        type: "Caddy",
      });

      if (res.data) {
        toast.success(`Domain ${fullDomain} registered successfully`);
        this.showRegisterDialog = false;
        await this.loadAll();
      } else {
        toast.error(res.error || "Failed to register domain");
      }
    } catch (err) {
      toast.error("Network error during registration");
    } finally {
      this.registrationLoading = false;
    }
  }

  async loadRemoteRecords(provider: string) {
    const team = get(activeTeam);
    if (!team?.team?.id) return;

    this.loadingRemote = true;
    try {
      const res = await api.dns.listRemoteRecords(team.team.id, provider);
      if (res.data) {
        // Filter out records that are already managed in Labuh
        const managedNames = new Set(this.domains.map((d) => d.domain));
        this.remoteRecords = res.data.filter((r) => !managedNames.has(r.name));
      }
    } catch (err) {
      console.error("Failed to load remote records", err);
    } finally {
      this.loadingRemote = false;
    }
  }

  async openImportDialog(record: RemoteDnsRecord) {
    const team = get(activeTeam);
    if (!team?.team?.id) return;

    this.selectedRemoteRecord = record;
    this.selectedBaseDomain = record.zone_name;
    const parts = record.name.split(".");
    if (parts.length > record.zone_name.split(".").length) {
      this.subdomain = record.name.replace(`.${record.zone_name}`, "");
    } else {
      this.subdomain = "";
    }

    this.showRegisterDialog = true;
    this.loading = true;

    try {
      const stacksRes = await api.stacks.list(team.team.id);
      if (stacksRes.data) {
        this.stacks = stacksRes.data;
      }
    } finally {
      this.loading = false;
    }
  }
}
