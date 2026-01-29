import { browser } from "$app/environment";
import { auth, type User } from "$lib/stores";

export const API_URL = browser
  ? import.meta.env.PUBLIC_API_URL || "http://localhost:3000"
  : "";

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

export interface Stack {
  id: string;
  name: string;
  user_id: string;
  compose_content?: string;
  status: string;
  webhook_token?: string;
  container_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateStack {
  name: string;
  compose_content: string;
}

export interface Domain {
  id: string;
  stack_id: string;
  container_name: string;
  container_port: number;
  domain: string;
  ssl_enabled: boolean;
  verified: boolean;
  created_at: string;
}

export interface CreateDomain {
  domain: string;
  container_name: string;
  container_port?: number;
}

export interface RegistryCredential {
  id: string;
  name: string;
  registry_url: string;
  username: string;
  created_at: string;
  updated_at: string;
}

export interface CreateRegistryCredential {
  name: string;
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
    list: async (all = false) => {
      return fetchApi<Container[]>(`/containers?all=${all}`);
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

    pull: async (image: string) => {
      return fetchApi<{ status: string; image: string }>("/images/pull", {
        method: "POST",
        body: JSON.stringify({ image }),
      });
    },

    remove: async (id: string) => {
      return fetchApi<{ status: string }>(`/images/${id}`, {
        method: "DELETE",
      });
    },
  },

  stacks: {
    list: async () => {
      return fetchApi<Stack[]>("/stacks");
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

    regenerateWebhookToken: async (id: string) => {
      return fetchApi<{ token: string }>(`/stacks/${id}/webhook/regenerate`, {
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
  },

  registries: {
    list: async () => {
      return fetchApi<RegistryCredential[]>("/registries");
    },

    add: async (data: CreateRegistryCredential) => {
      return fetchApi<RegistryCredential>("/registries", {
        method: "POST",
        body: JSON.stringify(data),
      });
    },

    remove: async (id: string) => {
      return fetchApi<{ status: string }>(`/registries/${id}`, {
        method: "DELETE",
      });
    },
  },
};
