<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { RefreshCw } from '@lucide/svelte';
  import type { ContainerController } from '../container-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: ContainerController }>();

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
</script>

{#if ctrl.stats && ctrl.container?.state === 'running'}
<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <Card.Title>Resource Usage</Card.Title>
      <Button variant="ghost" size="sm" onclick={() => ctrl.loadStats()}>
        <RefreshCw class="h-4 w-4" />
      </Button>
    </div>
  </Card.Header>
  <Card.Content>
    <div class="grid gap-4 md:grid-cols-4">
      <div>
        <p class="text-sm text-muted-foreground">CPU</p>
        <p class="text-2xl font-bold">{ctrl.stats.cpu_percent.toFixed(1)}%</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Memory</p>
        <p class="text-2xl font-bold">{ctrl.stats.memory_percent.toFixed(1)}%</p>
        <p class="text-xs text-muted-foreground">
          {formatBytes(ctrl.stats.memory_usage)} / {formatBytes(ctrl.stats.memory_limit)}
        </p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Network RX</p>
        <p class="text-2xl font-bold">{formatBytes(ctrl.stats.network_rx)}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Network TX</p>
        <p class="text-2xl font-bold">{formatBytes(ctrl.stats.network_tx)}</p>
      </div>
    </div>
  </Card.Content>
</Card.Root>
{/if}
