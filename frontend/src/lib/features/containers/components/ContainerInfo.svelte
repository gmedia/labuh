<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Play, Square, RotateCcw, Trash2 } from '@lucide/svelte';
  import type { ContainerController } from '../container-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: ContainerController }>();

  function getStatusColor(state: string | undefined): string {
    switch (state) {
      case 'running': return 'bg-green-500';
      case 'exited': return 'bg-red-500';
      case 'paused': return 'bg-yellow-500';
      default: return 'bg-muted-foreground';
    }
  }
</script>

{#if ctrl.container}
<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <span class="h-3 w-3 rounded-full {getStatusColor(ctrl.container.state)}"></span>
        <Card.Title>{ctrl.container.names[0]?.replace(/^\//, '') || ctrl.container.id.slice(0, 12)}</Card.Title>
      </div>
      <div class="flex items-center gap-2">
        {#if ctrl.container.state !== 'running'}
          <Button variant="outline" size="sm" onclick={() => ctrl.start()} disabled={ctrl.actionLoading}>
            <Play class="h-4 w-4 mr-1" /> Start
          </Button>
        {:else}
          <Button variant="outline" size="sm" onclick={() => ctrl.stop()} disabled={ctrl.actionLoading}>
            <Square class="h-4 w-4 mr-1" /> Stop
          </Button>
        {/if}
        <Button variant="outline" size="sm" onclick={() => ctrl.restart()} disabled={ctrl.actionLoading}>
          <RotateCcw class="h-4 w-4 mr-1" /> Restart
        </Button>
        <Button variant="outline" size="sm" onclick={() => ctrl.remove()} disabled={ctrl.actionLoading}>
          <Trash2 class="h-4 w-4 text-destructive" />
        </Button>
      </div>
    </div>
  </Card.Header>
  <Card.Content>
    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <div>
        <p class="text-sm text-muted-foreground">Image</p>
        <p class="font-medium">{ctrl.container.image}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Status</p>
        <p class="font-medium capitalize">{ctrl.container.state} - {ctrl.container.status}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Container ID</p>
        <p class="font-medium font-mono text-sm">{ctrl.container.id.slice(0, 12)}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Created</p>
        <p class="font-medium">{new Date(ctrl.container.created * 1000).toLocaleString()}</p>
      </div>
    </div>
    {#if ctrl.container.ports && ctrl.container.ports.length > 0}
      <div class="mt-4">
        <p class="text-sm text-muted-foreground mb-1">Ports</p>
        <div class="flex gap-2 text-wrap">
          {#each ctrl.container.ports as port}
            <span class="px-2 py-1 bg-muted rounded text-sm">
              {port.public_port || '?'}:{port.private_port}/{port.port_type}
            </span>
          {/each}
        </div>
      </div>
    {/if}
  </Card.Content>
</Card.Root>
{/if}
