<script lang="ts">
  import { onMount } from "svelte";
  import { NetworkController } from "$lib/features/networks/network-controller.svelte";
  import NetworkVisualizer from "$lib/features/networks/components/NetworkVisualizer.svelte";
  import { Network, Search, Filter, Info, RefreshCw } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import * as Tabs from "$lib/components/ui/tabs";
  import { Input } from "$lib/components/ui/input";

  let ctrl = $state(new NetworkController());

  onMount(async () => {
    await ctrl.init();
  });
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h2 class="text-2xl font-bold tracking-tight">Network Visualization</h2>
      <p class="text-muted-foreground text-sm">Explore container connectivity and network topology across the cluster</p>
    </div>
  </div>

  <div class="grid gap-6">
    <Card.Root>
      <Card.Header class="pb-3 border-b bg-muted/20">
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
                <div class="bg-primary/10 p-2 rounded-lg">
                    <Network class="h-5 w-5 text-primary" />
                </div>
                <div>
                    <Card.Title>Topology Graph</Card.Title>
                    <Card.Description>Interactive force-directed visualization</Card.Description>
                </div>
            </div>
            <div class="flex items-center gap-2">
                <div class="hidden md:flex flex-col text-right mr-4">
                    <span class="text-xs font-medium">{ctrl.topology.nodes.length} Nodes</span>
                    <span class="text-[10px] text-muted-foreground">{ctrl.topology.edges.length} Edges</span>
                </div>
                <Button variant="outline" size="sm" onclick={() => ctrl.loadTopology()} disabled={ctrl.loading}>
                    <RefreshCw class="h-3.5 w-3.5 mr-2 {ctrl.loading ? 'animate-spin' : ''}" />
                    Refresh
                </Button>
            </div>
        </div>
      </Card.Header>
      <Card.Content class="p-0">
        <NetworkVisualizer {ctrl} />
      </Card.Content>
      <Card.Footer class="bg-muted/10 border-t p-4">
        <div class="flex items-center gap-1 text-xs text-muted-foreground">
            <Info class="h-3.5 w-3.5" />
            <span>Click and drag nodes to rearrange. Use mouse wheel or buttons to zoom.</span>
        </div>
      </Card.Footer>
    </Card.Root>

    <div class="grid md:grid-cols-3 gap-6">
        <Card.Root>
            <Card.Header>
                <Card.Title class="text-sm font-medium">Quick Stats</Card.Title>
            </Card.Header>
            <Card.Content>
                <dl class="space-y-4">
                    <div class="flex justify-between items-center">
                        <dt class="text-xs text-muted-foreground">Total Networks</dt>
                        <dd class="text-sm font-semibold">{ctrl.topology.nodes.filter(n => n.type === 'network').length}</dd>
                    </div>
                    <div class="flex justify-between items-center">
                        <dt class="text-xs text-muted-foreground">Total Containers</dt>
                        <dd class="text-sm font-semibold">{ctrl.topology.nodes.filter(n => n.type === 'container').length}</dd>
                    </div>
                    <div class="flex justify-between items-center">
                        <dt class="text-xs text-muted-foreground">Active Connections</dt>
                        <dd class="text-sm font-semibold">{ctrl.topology.edges.length}</dd>
                    </div>
                </dl>
            </Card.Content>
        </Card.Root>

        <Card.Root class="md:col-span-2">
            <Card.Header>
                <Card.Title class="text-sm font-medium">Connectivity Insights</Card.Title>
            </Card.Header>
            <Card.Content>
                <div class="text-xs text-muted-foreground">
                    {#if ctrl.topology.nodes.length > 0}
                        <p>Most connected network: <span class="text-foreground font-medium">
                            {(() => {
                                const networkCounts = ctrl.topology.edges.reduce((acc, edge) => {
                                    acc[edge.to] = (acc[edge.to] || 0) + 1;
                                    return acc;
                                }, {} as Record<string, number>);
                                const topNetworkId = Object.entries(networkCounts).sort((a,b) => b[1] - a[1])[0]?.[0];
                                return ctrl.topology.nodes.find(n => n.id === topNetworkId)?.label || 'None';
                            })()}
                        </span></p>
                    {:else}
                        <p>No data available</p>
                    {/if}
                </div>
            </Card.Content>
        </Card.Root>
    </div>
  </div>
</div>
