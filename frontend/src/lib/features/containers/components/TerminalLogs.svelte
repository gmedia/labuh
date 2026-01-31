<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Terminal, RefreshCw } from '@lucide/svelte';
  import type { ContainerController } from '../container-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: ContainerController }>();
</script>

<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Terminal class="h-5 w-5" />
        <Card.Title>Logs</Card.Title>
      </div>
      <Button variant="outline" size="sm" onclick={() => ctrl.loadLogs()} disabled={ctrl.logsLoading}>
        <RefreshCw class="h-4 w-4 mr-1 {ctrl.logsLoading ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    </div>
  </Card.Header>
  <Card.Content>
    <div class="bg-black rounded-lg p-4 max-h-96 overflow-auto font-mono text-sm text-green-400">
      {#if ctrl.logs.length === 0}
        <p class="text-muted-foreground">No logs available</p>
      {:else}
        {#each ctrl.logs as line}
          <div class="whitespace-pre-wrap break-all">{line}</div>
        {/each}
      {/if}
    </div>
  </Card.Content>
</Card.Root>
