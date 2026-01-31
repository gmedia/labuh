<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Container as ContainerIcon, Play, Square, RotateCcw, Trash2 } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { Container } from '$lib/api';
  import type { ContainerListController } from '../container-list-controller.svelte';

  let { container, ctrl } = $props<{ container: Container, ctrl: ContainerListController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  function getStatusColor(state: string): string {
    switch (state) {
      case 'running': return 'text-green-500';
      case 'exited': return 'text-red-500';
      case 'paused': return 'text-yellow-500';
      default: return 'text-muted-foreground';
    }
  }
</script>

<Card.Root>
  <Card.Content class="flex items-center justify-between p-4 text-wrap">
    <div class="flex items-center gap-4 overflow-hidden">
      <ContainerIcon class="h-8 w-8 text-muted-foreground flex-shrink-0" />
      <div class="overflow-hidden">
        <a href="/dashboard/containers/{container.id}" class="hover:underline flex flex-col">
          <span class="font-medium truncate">{container.names[0]?.replace(/^\//, '') || container.id.slice(0, 12)}</span>
          <span class="text-sm text-muted-foreground truncate">{container.image}</span>
          <span class="text-xs {getStatusColor(container.state)} capitalize">{container.state} - {container.status}</span>
        </a>
      </div>
    </div>
    <div class="flex items-center gap-2 flex-shrink-0">
      {#if !isViewer}
        {#if container.state !== 'running'}
          <Button variant="outline" size="icon" onclick={() => ctrl.startContainer(container.id)} disabled={ctrl.actionLoading === container.id}>
            <Play class="h-4 w-4" />
          </Button>
        {:else}
          <Button variant="outline" size="icon" onclick={() => ctrl.stopContainer(container.id)} disabled={ctrl.actionLoading === container.id}>
            <Square class="h-4 w-4" />
          </Button>
        {/if}
        <Button variant="outline" size="icon" onclick={() => ctrl.restartContainer(container.id)} disabled={ctrl.actionLoading === container.id}>
          <RotateCcw class="h-4 w-4" />
        </Button>
        <Button variant="outline" size="icon" onclick={() => ctrl.removeContainer(container.id)} disabled={ctrl.actionLoading === container.id}>
          <Trash2 class="h-4 w-4 text-destructive" />
        </Button>
      {:else}
        <span class="text-xs text-muted-foreground italic px-2">Read Only</span>
      {/if}
    </div>
  </Card.Content>
</Card.Root>
