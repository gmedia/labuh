<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Terminal, RefreshCw } from '@lucide/svelte';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();

  const selectedContainerName = $derived(
    ctrl.containers.find((c: any) => c.id === ctrl.selectedContainerLogs)?.names[0]?.replace(/^\//, '') ||
    ctrl.selectedContainerLogs?.slice(0, 12)
  );
</script>

{#if ctrl.selectedContainerLogs}
<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <Card.Title class="flex items-center gap-2">
        <Terminal class="h-5 w-5" />
        Logs: {selectedContainerName}
      </Card.Title>
      <div class="flex gap-2">
        <Button variant="ghost" size="sm" onclick={() => ctrl.loadContainerLogs(ctrl.selectedContainerLogs!)}>
          <RefreshCw class="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="sm" onclick={() => ctrl.selectedContainerLogs = null}>
          Close
        </Button>
      </div>
    </div>
  </Card.Header>
  <Card.Content>
    <div class="bg-black rounded-lg p-4 max-h-80 overflow-auto font-mono text-sm text-green-400">
      {#if !ctrl.logs.get(ctrl.selectedContainerLogs) || ctrl.logs.get(ctrl.selectedContainerLogs)?.length === 0}
        <p class="text-muted-foreground">No logs available</p>
      {:else}
        {#each ctrl.logs.get(ctrl.selectedContainerLogs) || [] as line}
          <div class="whitespace-pre-wrap break-all">{line}</div>
        {/each}
      {/if}
    </div>
  </Card.Content>
</Card.Root>
{/if}
