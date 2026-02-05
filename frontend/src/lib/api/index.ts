import { browser, dev } from "$app/environment";
import { auth, type User } from "$lib/stores";

export const API_URL =
  import.meta.env.PUBLIC_API_URL ||
  (browser && !dev ? "" : "http://localhost:3000");

interface ApiResponse<T> {
  data?: T;
  error?: string;
  message?: string;
}

async function fetchApi<T>(
  endpoint: string,
  options: RequestInit = {},
): Promise<ApiResponse<T>> {
  const token = browser ? localStorage.getItem("token") : null;

  const headers: HeadersInit = {
    "Content-Type": "application/json",
    ...(options.headers || {}),
  };

  if (token) {
    (headers as Record<string, string>)["Authorization"] = `Bearer ${token}`;
  }

  try {
    const response = await fetch(`${API_URL}/api${endpoint}`, {
      ...options,
      headers,
    });

    const data = await response.json();

    if (!response.ok) {
      // Auto-logout if user no longer exists (e.g., database was reset)
      if (response.status === 401 && data.error === "user_not_found") {
        localStorage.removeItem("token");
        localStorage.removeItem("user");
        window.location.href = "/login";
      }
      return { error: data.error || "Request failed", message: data.message };
    }

    return { data };
  } catch (error) {
    return { error: "Network error", message: String(error) };
  }
}

// Types
export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name?: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface Container {
  id: string;
  names: string[];
  image: string;
  state: string;
  status: string;
  ports: { private_port: number; public_port?: number; port_type: string }[];
  created: number;
  labels: Record<string, string>;
}

export interface ContainerStats {
  cpu_percent: number;
  memory_usage: number;
  memory_limit: number;
  memory_percent: number;
  network_rx: number;
  network_tx: number;
}

export interface Image {
  id: string;
  repo_tags: string[];
  size: number;
  created: number;
}

export interface ImageInspect {
  id: string;
  repo_tags: string[];
  exposed_ports: string[];
  env_vars: string[];
  working_dir: string;
  entrypoint: string[];
  cmd: string[];
  created: string;
  size: number;
}

export interface DeploymentLog {
  id: string;
  stack_id: string;
  trigger_type: string;
  status: string;
  logs?: string;
  started_at: string;
  finished_at?: string;
}

export interface SystemStats {
  cpu_count: number;
  memory_total_kb: number;
  memory_available_kb: number;
  memory_used_percent: number;
  disk_total_bytes: number;
  disk_available_bytes: number;
  disk_used_percent: number;
  uptime_seconds: number;
  load_average: { one: number; five: number; fifteen: number };
}

export type TeamRole = "Owner" | "Admin" | "Developer" | "Viewer";

export interface Team {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface TeamMember {
  team_id: string;
  user_id: string;
  user_name: string;
  user_email: string;
  role: TeamRole;
  created_at: string;
  updated_at: string;
}

export interface TeamResponse {
  team: Team;
  role: TeamRole;
}

export interface Stack {
  id: string;
  name: string;
  user_id: string;
  team_id: string;
  compose_content?: string;
  status: string;
  webhook_token?: string;
  cron_schedule?: string;
  health_check_path?: string;
  health_check_interval: number;
  last_stable_images?: string;
  git_url?: string;
  git_branch?: string;
  last_commit_hash?: string;
  container_count: number;
  created_at: string;
  updated_at: string;
}

export interface StackBackup {
  name: string;
  compose_content: string;
  env_vars: BackupEnvVar[];
}

export interface BackupEnvVar {
  container_name: string;
  key: string;
  value: string;
  is_secret: boolean;
}

export interface CreateStack {
  name: string;
  team_id: string;
  compose_content: string;
}

export interface CreateStackFromGit {
  name: string;
  team_id: string;
  git_url: string;
  git_branch: string;
  compose_path: string;
  env_vars?: Record<string, string>;
}

export type DomainProvider = "Custom" | "Cloudflare" | "CPanel";
export type DomainType = "Caddy" | "Tunnel";

export interface RemoteDnsRecord {
  id: string;
  name: string;
  content: string;
  type: string;
  zone_id: string;
  zone_name: string;
}

export interface Domain {
  id: string;
  stack_id: string;
  container_name: string;
  container_port: number;
  domain: string;
  ssl_enabled: boolean;
  verified: boolean;
  provider: DomainProvider;
  type: DomainType;
  tunnel_id?: string;
  dns_record_id?: string;
  proxied: boolean;
  show_branding: boolean;
  created_at: string;
}

export interface CreateDomain {
  domain: string;
  container_name: string;
  container_port?: number;
  provider?: DomainProvider;
  type?: DomainType;
  tunnel_id?: string;
  tunnel_token?: string;
  dns_record_type?: string;
  dns_record_content?: string;
  proxied?: boolean;
}

export interface DnsConfig {
  id: string;
  team_id: string;
  provider: string;
  config: any;
  created_at: string;
  updated_at: string;
}

export interface RegistryCredential {
  id: string;
  user_id: string;
  team_id: string;
  name: string;
  registry_url: string;
  username: string;
  created_at: string;
  updated_at: string;
}

export interface CreateRegistryCredential {
  name: string;
  team_id: string;
  registry_url: string;
  username: string;
  password: string;
}

export interface ContainerHealth {
  container_id: string;
  name: string;
  state: string;
  status: string;
}

export interface StackHealth {
  stack_id: string;
  stack_name: string;
  status: string;
  containers: ContainerHealth[];
  healthy_count: number;
  total_count: number;
}

export interface StackLogEntry {
  container_name: string;
  line: string;
}

export interface EnvVar {
  id: string;
  stack_id: string;
  container_name: string;
  key: string;
  value: string;
  is_secret: boolean;
  created_at: string;
  updated_at: string;
}

export interface SetEnvVar {
  container_name: string;
  key: string;
  value: string;
  is_secret?: boolean;
}

export interface ContainerResource {
  id: string;
  stack_id: string;
  service_name: string;
  cpu_limit?: number;
  memory_limit?: number;
  created_at: string;
  updated_at: string;
}

export interface ResourceMetric {
  id: string;
  container_id: string;
  stack_id: string;
  cpu_usage: number;
  memory_usage: number;
  timestamp: string;
}

export interface HistoricalNodeMetrics {
  cpu_percent: number;
  memory_usage: number;
  memory_total: number;
  disk_usage: number;
  disk_total: number;
  timestamp: string;
}

export interface HistoricalContainerMetrics {
  container_id: string;
  stack_id: string;
  cpu_percent: number;
  memory_usage: number;
  memory_limit: number;
  timestamp: string;
}

export interface Template {
  id: string;
  name: string;
  description: string;
  icon: string;
  compose_content: string;
  default_env: TemplateEnv[];
}

export interface TemplateEnv {
  key: string;
  value: string;
  description?: string;
}

export interface TemplateResponse {
  id: string;
  name: string;
  description: string;
  icon: string;
}

export interface SwarmNode {
  id: string;
  hostname: string;
  role: string;
  status: string;
  availability: string;
  addr: string;
  version: string;
  platform: string;
  resources: {
    nano_cpus: number;
    memory_bytes: number;
  };
}

export const api = {
  auth: {
    login: async (data: LoginRequest) => {
      const result = await fetchApi<AuthResponse>("/auth/login", {
        method: "POST",
        body: JSON.stringify(data),
      });
      if (result.data) {
        auth.login(result.data.token, result.data.user);
      }
      return result;
    },

    register: async (data: RegisterRequest) => {
      const result = await fetchApi<AuthResponse>("/auth/register", {
        method: "POST",
        body: JSON.stringify(data),
      });
      if (result.data) {
        auth.login(result.data.token, result.data.user);
      }
      return result;
    },

    logout: () => {
      auth.logout();
    },

    me: async () => {
      return fetchApi<User>("/me");
    },

    isSetupRequired: async () => {
      return fetchApi<boolean>("/auth/setup-required");
    },
  },

  health: {
    check: async () => {
      return fetchApi<{ status: string; version: string }>("/health");
    },
  },

  system: {
    stats: async () => {
      return fetchApi<SystemStats>("/system/stats");
    },
  },

  containers: {
    list: async (all = false, teamId?: string) => {
      const query = new URLSearchParams({ all: all.toString() });
      if (teamId) query.append("team_id", teamId);
      return fetchApi<Container[]>(`/containers?${query.toString()}`);
    },

    create: async (data: {
      name: string;
      image: string;
      env?: string[];
      ports?: Record<string, string>;
      volumes?: Record<string, string>;
    }) => {
      return fetchApi<{ id: string }>("/containers", {
        method: "POST",
        body: JSON.stringify(data),
      });
    },

    start: async (id: string) => {
      return fetchApi<{ status: string }>(`/containers/${id}/start`, {
        method: "POST",
      });
    },

    stop: async (id: string) => {
      return fetchApi<{ status: string }>(`/containers/${id}/stop`, {
        method: "POST",
      });
    },

    restart: async (id: string) => {
      return fetchApi<{ status: string }>(`/containers/${id}/restart`, {
        method: "POST",
      });
    },

    remove: async (id: string) => {
      return fetchApi<{ status: string }>(`/containers/${id}`, {
        method: "DELETE",
      });
    },

    logs: async (id: string, tail = 100) => {
      return fetchApi<string[]>(`/containers/${id}/logs?tail=${tail}`);
    },

    stats: async (id: string) => {
      return fetchApi<ContainerStats>(`/containers/${id}/stats`);
    },
  },

  images: {
    list: async () => {
      return fetchApi<Image[]>("/images");
    },

    inspect: async (id: string) => {
      return fetchApi<ImageInspect>(
        `/images/${encodeURIComponent(id)}/inspect`,
      );
    },

    pull: async (image: string, teamId: string) => {
      return fetchApi<{ status: string; image: string }>("/images/pull", {
        method: "POST",
        body: JSON.stringify({ image, team_id: teamId }),
      });
    },

    remove: async (id: string) => {
      return fetchApi<{ status: string }>(`/images/${id}`, {
        method: "DELETE",
      });
    },
  },

  stacks: {
    list: async (teamId: string) => {
      return fetchApi<Stack[]>(`/stacks?team_id=${teamId}`);
    },

    get: async (id: string) => {
      return fetchApi<Stack>(`/stacks/${id}`);
    },

    create: async (data: CreateStack) => {
      return fetchApi<Stack>("/stacks", {
        method: "POST",
        body: JSON.stringify(data),
      });
    },

    containers: async (id: string) => {
      return fetchApi<Container[]>(`/stacks/${id}/containers`);
    },

    start: async (id: string) => {
      return fetchApi<{ status: string }>(`/stacks/${id}/start`, {
        method: "POST",
      });
    },

    stop: async (id: string) => {
      return fetchApi<{ status: string }>(`/stacks/${id}/stop`, {
        method: "POST",
      });
    },

    redeploy: async (id: string, serviceName?: string) => {
      const url = serviceName
        ? `/stacks/${id}/services/${serviceName}/redeploy`
        : `/stacks/${id}/redeploy`;
      return fetchApi<{ status: string }>(url, {
        method: "POST",
      });
    },

    scale: async (id: string, serviceName: string, replicas: number) => {
      return fetchApi<{ status: string }>(
        `/stacks/${id}/services/${serviceName}/scale`,
        {
          method: "POST",
          body: JSON.stringify({ replicas }),
        },
      );
    },

    build: async (id: string, serviceName?: string) => {
      const url = serviceName
        ? `/stacks/${id}/services/${serviceName}/build`
        : `/stacks/${id}/build`;
      return fetchApi<{ status: string }>(url, {
        method: "POST",
      });
    },

    updateCompose: async (id: string, composeContent: string) => {
      return fetchApi<{ status: string }>(`/stacks/${id}/compose`, {
        method: "PUT",
        body: JSON.stringify({ compose_content: composeContent }),
      });
    },

    remove: async (id: string) => {
      return fetchApi<{ status: string }>(`/stacks/${id}`, {
        method: "DELETE",
      });
    },

    backup: async (id: string) => {
      return fetchApi<StackBackup>(`/stacks/${id}/backup`);
    },

    restore: async (teamId: string, backup: StackBackup) => {
      return fetchApi<Stack>("/stacks/restore", {
        method: "POST",
        body: JSON.stringify({ team_id: teamId, backup }),
      });
    },

    createFromGit: async (request: CreateStackFromGit) => {
      return fetchApi<Stack>("/stacks/git", {
        method: "POST",
        body: JSON.stringify(request),
      });
    },

    syncGit: async (id: string) => {
      return fetchApi<{ status: string }>(`/stacks/${id}/git/sync`, {
        method: "POST",
      });
    },

    regenerateWebhookToken: async (id: string) => {
      return fetchApi<{ token: string }>(`/stacks/${id}/webhook/regenerate`, {
        method: "POST",
      });
    },

    updateAutomation: async (
      id: string,
      data: {
        cron_schedule?: string;
        health_check_path?: string;
        health_check_interval: number;
      },
    ) => {
      return fetchApi<{ status: string }>(`/stacks/${id}/automation`, {
        method: "PUT",
        body: JSON.stringify(data),
      });
    },

    rollback: async (id: string) => {
      return fetchApi<{ status: string }>(`/stacks/${id}/rollback`, {
        method: "POST",
      });
    },

    deploymentLogs: async (id: string) => {
      return fetchApi<DeploymentLog[]>(`/stacks/${id}/deployments`);
    },

    domains: {
      list: async (stackId: string) => {
        return fetchApi<Domain[]>(`/stacks/${stackId}/domains`);
      },

      add: async (stackId: string, data: CreateDomain) => {
        return fetchApi<Domain>(`/stacks/${stackId}/domains`, {
          method: "POST",
          body: JSON.stringify(data),
        });
      },

      remove: async (stackId: string, domain: string) => {
        return fetchApi<{ status: string }>(
          `/stacks/${stackId}/domains/${encodeURIComponent(domain)}`,
          {
            method: "DELETE",
          },
        );
      },

      verify: async (stackId: string, domain: string) => {
        return fetchApi<{
          verified: boolean;
          a_records?: string[];
          cname_records?: string[];
        }>(`/stacks/${stackId}/domains/${encodeURIComponent(domain)}/verify`, {
          method: "POST",
        });
      },

      updateDns: async (
        stackId: string,
        domain: string,
        recordType: string,
        content: string,
        proxied: boolean,
      ) => {
        return fetchApi<{ status: string }>(
          `/stacks/${stackId}/domains/${encodeURIComponent(domain)}/dns`,
          {
            method: "PUT",
            body: JSON.stringify({
              record_type: recordType,
              content,
              proxied,
            }),
          },
        );
      },

      listAll: async (teamId: string) => {
        return fetchApi<Domain[]>(`/stacks/domains?team_id=${teamId}`);
      },

      syncAll: async () => {
        return fetchApi<{ status: string }>("/domains/sync", {
          method: "POST",
        });
      },

      toggleBranding: async (
        stackId: string,
        domain: string,
        showBranding: boolean,
      ) => {
        return fetchApi<{ status: string; show_branding: boolean }>(
          `/stacks/${stackId}/domains/${encodeURIComponent(domain)}/branding`,
          {
            method: "PUT",
            body: JSON.stringify({ show_branding: showBranding }),
          },
        );
      },
    },

    // Stack health overview
    health: async (id: string) => {
      return fetchApi<StackHealth>(`/stacks/${id}/health`);
    },

    // Combined logs from all containers
    logs: async (id: string, tail = 100) => {
      return fetchApi<StackLogEntry[]>(`/stacks/${id}/logs?tail=${tail}`);
    },

    // Environment variables
    env: {
      list: async (stackId: string) => {
        return fetchApi<EnvVar[]>(`/stacks/${stackId}/env`);
      },

      set: async (stackId: string, data: SetEnvVar) => {
        return fetchApi<EnvVar>(`/stacks/${stackId}/env`, {
          method: "POST",
          body: JSON.stringify(data),
        });
      },

      bulkSet: async (
        stackId: string,
        vars: { key: string; value: string; is_secret?: boolean }[],
        containerName = "",
      ) => {
        return fetchApi<EnvVar[]>(`/stacks/${stackId}/env/bulk`, {
          method: "PUT",
          body: JSON.stringify({ vars, container_name: containerName }),
        });
      },

      delete: async (stackId: string, key: string, containerName = "") => {
        return fetchApi<{ status: string }>(
          `/stacks/${stackId}/env/${encodeURIComponent(key)}?container_name=${encodeURIComponent(containerName)}`,
          {
            method: "DELETE",
          },
        );
      },
    },

    resources: {
      getLimits: async (stackId: string) => {
        return fetchApi<ContainerResource[]>(`/stacks/${stackId}/limits`);
      },

      updateLimits: async (
        stackId: string,
        serviceName: string,
        data: { cpu_limit?: number; memory_limit?: number },
      ) => {
        return fetchApi<{ status: string }>(
          `/stacks/${stackId}/services/${serviceName}/limits`,
          {
            method: "PUT",
            body: JSON.stringify(data),
          },
        );
      },

      getMetrics: async (stackId: string, range = "1h") => {
        return fetchApi<ResourceMetric[]>(
          `/stacks/${stackId}/metrics?range=${range}`,
        );
      },
    },
  },

  registries: {
    list: async (teamId: string) => {
      return fetchApi<RegistryCredential[]>(`/registries?team_id=${teamId}`);
    },

    add: async (data: CreateRegistryCredential) => {
      return fetchApi<RegistryCredential>("/registries", {
        method: "POST",
        body: JSON.stringify(data),
      });
    },

    remove: async (id: string, teamId: string) => {
      return fetchApi<{ status: string }>(`/registries/${teamId}/${id}`, {
        method: "DELETE",
      });
    },
  },

  teams: {
    list: async () => {
      return fetchApi<TeamResponse[]>("/teams");
    },

    create: async (name: string) => {
      return fetchApi<Team>("/teams", {
        method: "POST",
        body: JSON.stringify({ name }),
      });
    },

    remove: async (id: string) => {
      return fetchApi<{ status: string }>(`/teams/${id}`, {
        method: "DELETE",
      });
    },

    getMembers: async (id: string) => {
      return fetchApi<TeamMember[]>(`/teams/${id}/members`);
    },

    addMember: async (
      id: string,
      data: { name: string; email: string; password?: string; role: string },
    ) => {
      return fetchApi<{ status: string }>(`/teams/${id}/members`, {
        method: "POST",
        body: JSON.stringify(data),
      });
    },

    removeMember: async (teamId: string, userId: string) => {
      return fetchApi<{ status: string }>(
        `/teams/${teamId}/members/${userId}`,
        {
          method: "DELETE",
        },
      );
    },

    updateMemberRole: async (
      teamId: string,
      userId: string,
      role: TeamRole,
    ) => {
      return fetchApi<{ status: string }>(
        `/teams/${teamId}/members/${userId}/role`,
        {
          method: "PUT",
          body: JSON.stringify({ role }),
        },
      );
    },
  },

  templates: {
    list: async () => {
      return fetchApi<TemplateResponse[]>("/templates");
    },

    get: async (id: string) => {
      return fetchApi<Template>(`/templates/${id}`);
    },

    create: async (template: Template) => {
      return fetchApi<void>("/templates", {
        method: "POST",
        body: JSON.stringify(template),
      });
    },

    import: async (url: string) => {
      return fetchApi<Template>("/templates/import", {
        method: "POST",
        body: JSON.stringify({ url }),
      });
    },

    delete: async (id: string) => {
      return fetchApi<void>(`/templates/${id}`, {
        method: "DELETE",
      });
    },
  },

  dns: {
    listConfigs: async (teamId: string) => {
      return fetchApi<DnsConfig[]>(`/teams/${teamId}/dns-configs`);
    },

    saveConfig: async (teamId: string, provider: string, config: any) => {
      return fetchApi<DnsConfig>(`/teams/${teamId}/dns-configs`, {
        method: "POST",
        body: JSON.stringify({ provider, config }),
      });
    },

    deleteConfig: async (teamId: string, provider: string) => {
      return fetchApi<{ status: string }>(
        `/teams/${teamId}/dns-configs/${provider}`,
        {
          method: "DELETE",
        },
      );
    },
    listAvailableDomains: async (teamId: string, provider: string) => {
      return fetchApi<string[]>(
        `/teams/${teamId}/dns-configs/${provider}/available-domains`,
      );
    },
    listRemoteRecords: async (teamId: string, provider: string) => {
      return fetchApi<RemoteDnsRecord[]>(
        `/teams/${teamId}/dns-configs/${provider}/remote-records`,
      );
    },
  },

  nodes: {
    list: async () => {
      return fetchApi<SwarmNode[]>("/nodes");
    },

    get: async (id: string) => {
      return fetchApi<SwarmNode>(`/nodes/${id}`);
    },

    swarm: async () => {
      return fetchApi<{ enabled: boolean }>("/nodes/swarm");
    },

    initSwarm: async (listenAddr: string) => {
      return fetchApi<{ token: string }>("/nodes/swarm/init", {
        method: "POST",
        body: JSON.stringify({ listen_addr: listenAddr }),
      });
    },

    joinSwarm: async (data: {
      listen_addr: string;
      remote_addr: string;
      token: string;
    }) => {
      return fetchApi<{ status: string }>("/nodes/swarm/join", {
        method: "POST",
        body: JSON.stringify(data),
      });
    },

    getSwarmTokens: async () => {
      return fetchApi<{ manager: string; worker: string }>(
        "/nodes/swarm/tokens",
      );
    },
  },

  metrics: {
    getNodeMetrics: async (lastHours?: number) => {
      const query = lastHours ? `?last_hours=${lastHours}` : "";
      return fetchApi<HistoricalNodeMetrics[]>(
        `/metrics/nodes/metrics${query}`,
      );
    },

    getContainerMetrics: async (
      stackId: string,
      containerId: string,
      lastHours?: number,
    ) => {
      const query = lastHours ? `?last_hours=${lastHours}` : "";
      return fetchApi<HistoricalContainerMetrics[]>(
        `/metrics/stacks/${stackId}/containers/${containerId}/metrics${query}`,
      );
    },
  },

  networks: {
    getTopology: async () => {
      return fetchApi<{
        nodes: { id: string; label: string; type: string; metadata: any }[];
        edges: { from: string; to: string; label?: string }[];
      }>("/networks/topology");
    },
  },
};
