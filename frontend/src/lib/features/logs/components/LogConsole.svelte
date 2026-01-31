<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Terminal, Search, Download, RefreshCw } from '@lucide/svelte';
  import type { LogsController } from '../logs-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: LogsController }>();

  const containerName = $derived(
    ctrl.selectedContainer?.names[0]?.replace(/^\//, '') || 'Logs'
  );
</script>

<Card.Root class="lg:col-span-3">
  <Card.Header>
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2 text-wrap overflow-hidden">
        <Terminal class="h-5 w-5 flex-shrink-0" />
        <Card.Title class="truncate">
          {containerName}
        </Card.Title>
      </div>
      <div class="flex items-center gap-2 flex-shrink-0">
        <Button variant="outline" size="sm" onclick={() => ctrl.loadLogs()} disabled={ctrl.logsLoading}>
          <RefreshCw class="h-4 w-4 {ctrl.logsLoading ? 'animate-spin' : ''}" />
        </Button>
        <Button variant="outline" size="sm" onclick={() => ctrl.downloadLogs()} disabled={ctrl.logs.length === 0}>
          <Download class="h-4 w-4" />
        </Button>
      </div>
    </div>
    <div class="relative">
      <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
      <Input
        placeholder="Search logs..."
        class="pl-9"
        bind:value={ctrl.searchQuery}
      />
    </div>
  </Card.Header>
  <Card.Content>
    <div class="bg-black rounded-lg p-4 max-h-[500px] overflow-auto font-mono text-sm text-green-400">
      {#if ctrl.filteredLogs.length === 0}
        <p class="text-muted-foreground text-center py-8">
          {ctrl.searchQuery ? 'No matching logs found' : 'No logs available'}
        </p>
      {:else}
        {#each ctrl.filteredLogs as line, i}
          <div class="whitespace-pre-wrap break-all hover:bg-white/5 px-1 flex gap-3">
            <span class="text-muted-foreground mr-2 select-none flex-shrink-0 w-8 text-right">{i + 1}</span>
            <span>{line}</span>
          </div>
        {/each}
      {/if}
    </div>
    <div class="mt-2 text-xs text-muted-foreground flex justify-between">
      <span>Showing {ctrl.filteredLogs.length} of {ctrl.logs.length} lines</span>
      {#if ctrl.logsLoading}
         <span class="animate-pulse">Loading new logs...</span>
      {/if}
    </div>
  </Card.Content>
</Card.Root>
