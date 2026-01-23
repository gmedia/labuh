import { browser } from "$app/environment";
import { auth, type User } from "$lib/stores";

const API_URL = browser
  ? import.meta.env.PUBLIC_API_URL || "http://localhost:3000"
  : " ";

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

export interface Project {
  id: string;
  name: string;
  slug: string;
  description?: string;
  container_id?: string;
  image?: string;
  status: string;
  port?: number;
  env_vars?: Record<string, string>;
  domains?: string[];
  user_id: string;
  webhook_token?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateProject {
  name: string;
  description?: string;
  image?: string;
  port?: number;
  env_vars?: Record<string, string>;
  domains?: string[];
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
  project_id: string;
  domain: string;
  ssl_enabled: boolean;
  verified: boolean;
  created_at: string;
}

export interface CreateDomain {
  domain: string;
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

  projects: {
    list: async () => {
      return fetchApi<Project[]>("/projects");
    },

    get: async (id: string) => {
      return fetchApi<Project>(`/projects/${id}`);
    },

    create: async (data: CreateProject) => {
      return fetchApi<Project>("/projects", {
        method: "POST",
        body: JSON.stringify(data),
      });
    },

    update: async (id: string, data: Partial<CreateProject>) => {
      return fetchApi<Project>(`/projects/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      });
    },

    remove: async (id: string) => {
      return fetchApi<{ status: string }>(`/projects/${id}`, {
        method: "DELETE",
      });
    },

    deploy: async (id: string) => {
      return fetchApi<Project>(`/projects/${id}/deploy`, {
        method: "POST",
      });
    },

    stop: async (id: string) => {
      return fetchApi<Project>(`/projects/${id}/stop`, {
        method: "POST",
      });
    },

    restart: async (id: string) => {
      return fetchApi<Project>(`/projects/${id}/restart`, {
        method: "POST",
      });
    },

    // Domain management
    domains: {
      list: async (projectId: string) => {
        return fetchApi<Domain[]>(`/projects/${projectId}/domains`);
      },

      add: async (projectId: string, domain: string) => {
        return fetchApi<Domain>(`/projects/${projectId}/domains`, {
          method: "POST",
          body: JSON.stringify({ domain }),
        });
      },

      remove: async (projectId: string, domain: string) => {
        return fetchApi<{ status: string }>(
          `/projects/${projectId}/domains/${encodeURIComponent(domain)}`,
          {
            method: "DELETE",
          },
        );
      },

      verify: async (projectId: string, domain: string) => {
        return fetchApi<{ verified: boolean }>(
          `/projects/${projectId}/domains/${encodeURIComponent(domain)}/verify`,
          {
            method: "POST",
          },
        );
      },
    },

    regenerateWebhookToken: async (id: string) => {
      return fetchApi<Project>(`/projects/${id}/webhook/regenerate`, {
        method: "POST",
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

    remove: async (id: string) => {
      return fetchApi<{ status: string }>(`/stacks/${id}`, {
        method: "DELETE",
      });
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
