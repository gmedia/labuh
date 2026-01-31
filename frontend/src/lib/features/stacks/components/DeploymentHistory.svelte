<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { History, RefreshCw, CheckCircle, XCircle, RotateCcw } from '@lucide/svelte';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
</script>

<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <Card.Title class="flex items-center gap-2">
        <History class="h-5 w-5" />
        Deployments
      </Card.Title>
      <Button variant="ghost" size="sm" onclick={() => ctrl.loadDeployments()}>
        <RefreshCw class="h-3 w-3" />
      </Button>
    </div>
  </Card.Header>
  <Card.Content>
    {#if ctrl.deployments.length === 0}
      <p class="text-xs text-muted-foreground text-center py-4">No deployments yet</p>
    {:else}
      <div class="space-y-2 max-h-60 overflow-y-auto pr-1">
        {#each ctrl.deployments as log}
          <div class="p-2 rounded border bg-background/50 text-xs">
            <div class="flex items-center justify-between mb-1">
              <span class="font-medium capitalize flex items-center gap-1">
                {#if log.status === 'success'}
                  <CheckCircle class="h-3 w-3 text-green-500" />
                {:else if log.status === 'failed'}
                  <XCircle class="h-3 w-3 text-red-500" />
                {:else}
                  <RotateCcw class="h-3 w-3 text-yellow-500 animate-spin" />
                {/if}
                {log.trigger_type}
              </span>
              <span class="text-muted-foreground">{new Date(log.started_at).toLocaleDateString()}</span>
            </div>
            <div class="flex justify-between text-muted-foreground">
              <span>{new Date(log.started_at).toLocaleTimeString()}</span>
              <span class="capitalize">{log.status}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </Card.Content>
</Card.Root>
