import { api } from "$lib/api";
import { toast } from "svelte-sonner";

export interface TopologyNode {
  id: string;
  label: string;
  type: string;
  metadata: any;
  x?: number;
  y?: number;
}

export interface TopologyEdge {
  from: string;
  to: string;
  label?: string;
  source?: any;
  target?: any;
}

export class NetworkController {
  topology = $state<{ nodes: TopologyNode[]; edges: TopologyEdge[] }>({
    nodes: [],
    edges: [],
  });
  loading = $state(false);

  async init() {
    await this.loadTopology();
  }

  async loadTopology() {
    this.loading = true;
    try {
      const res = await api.networks.getTopology();
      if (res.data) {
        this.topology = res.data;
      } else {
        toast.error(res.error || "Failed to load network topology");
      }
    } catch (err) {
      toast.error("Network error while loading topology");
    } finally {
      this.loading = false;
    }
  }
}
