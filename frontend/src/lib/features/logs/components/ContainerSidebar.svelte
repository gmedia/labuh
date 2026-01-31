<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import type { LogsController } from '../logs-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: LogsController }>();

  function getContainerName(container: any): string {
    return container.names[0]?.replace(/^\//, '') || container.id.slice(0, 12);
  }
</script>

<Card.Root class="lg:col-span-1">
  <Card.Header class="pb-2">
    <Card.Title class="text-sm">Containers</Card.Title>
  </Card.Header>
  <Card.Content class="p-0">
    <div class="space-y-1 p-2">
      {#each ctrl.containers as container}
        <button
          class="w-full text-left px-3 py-2 rounded-lg transition-colors {ctrl.selectedContainerId === container.id ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
          onclick={() => ctrl.selectContainer(container.id)}
        >
          <p class="font-medium text-sm truncate">{getContainerName(container)}</p>
          <p class="text-xs opacity-75 capitalize">{container.state}</p>
        </button>
      {/each}
    </div>
  </Card.Content>
</Card.Root>
