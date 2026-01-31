<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Activity } from '@lucide/svelte';
  import ResourceChart from '$lib/components/ResourceChart.svelte';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
</script>

<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <Card.Title class="flex items-center gap-2">
        <Activity class="h-5 w-5" />
        Monitoring
      </Card.Title>
      <select bind:value={ctrl.selectedRange} class="bg-background border rounded px-2 py-1 text-xs">
        <option value="1h">Last Hour</option>
        <option value="6h">Last 6 Hours</option>
        <option value="24h">Last 24 Hours</option>
        <option value="7d">Last 7 Days</option>
      </select>
    </div>
  </Card.Header>
  <Card.Content class="space-y-6">
    {#if ctrl.metrics.length === 0}
      <p class="text-sm text-muted-foreground text-center py-8">No monitoring data available yet.</p>
    {:else}
      <div class="space-y-4">
        <div>
          <h4 class="text-xs font-medium uppercase text-muted-foreground mb-1">CPU Usage (%)</h4>
          <ResourceChart metrics={ctrl.metrics} type="cpu" />
        </div>
        <div>
          <h4 class="text-xs font-medium uppercase text-muted-foreground mb-1">Memory Usage (MB)</h4>
          <ResourceChart metrics={ctrl.metrics} type="memory" />
        </div>
      </div>
    {/if}
  </Card.Content>
</Card.Root>
