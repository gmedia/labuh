<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Container as ContainerIcon, RefreshCw, Terminal } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  function getStatusBg(state: string): string {
    switch (state) {
      case 'running': return 'bg-green-500';
      case 'exited': return 'bg-red-500';
      case 'paused': return 'bg-yellow-500';
      default: return 'bg-muted-foreground';
    }
  }
</script>

<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <Card.Title class="flex items-center gap-2">
        <ContainerIcon class="h-5 w-5" />
        Containers
      </Card.Title>
      <Button variant="ghost" size="sm" onclick={() => ctrl.loadContainers()}>
        <RefreshCw class="h-4 w-4" />
      </Button>
    </div>
  </Card.Header>
  <Card.Content class="space-y-2">
    {#if ctrl.containers.length === 0}
      <p class="text-sm text-muted-foreground text-center py-4">No containers in this stack</p>
    {:else}
      {#each ctrl.containers as container}
        <div class="flex items-center justify-between p-3 rounded-lg bg-muted/50 hover:bg-muted transition-colors">
          <div class="flex items-center gap-3">
            <span class="h-2 w-2 rounded-full {getStatusBg(container.state)}"></span>
            <div>
              <a href="/dashboard/containers/{container.id}" class="font-medium hover:underline">
                {container.names[0]?.replace(/^\//, '') || container.id.slice(0, 12)}
              </a>
              <div class="flex items-center gap-2 text-xs text-muted-foreground">
                <span>{container.image}</span>
                {#if container.ports && container.ports.length > 0}
                  <span>•</span>
                  <span>{container.ports.map((p: any) => `${p.public_port || p.private_port}`).join(', ')}</span>
                {/if}
              </div>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <Button variant="ghost" size="sm" onclick={() => ctrl.loadContainerLogs(container.id)} title="View Logs">
              <Terminal class="h-4 w-4" />
            </Button>
            {#if !isViewer}
              <Button
                variant="ghost"
                size="sm"
                onclick={() => ctrl.redeploy(container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, ''))}
                title="Redeploy Service"
                disabled={ctrl.actionLoading}
              >
                <RefreshCw class="h-4 w-4 {ctrl.actionLoading ? 'animate-spin' : ''}" />
              </Button>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </Card.Content>
</Card.Root>
