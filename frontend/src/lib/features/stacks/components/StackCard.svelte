<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Layers, Play, Square, Trash2 } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { Stack } from '$lib/api';
  import type { StackListController } from '../stack-list-controller.svelte';

  let { stack, ctrl } = $props<{ stack: Stack, ctrl: StackListController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  function getStatusColor(status: string): string {
    switch (status) {
      case 'running': return 'text-green-500';
      case 'stopped': return 'text-red-500';
      case 'creating': return 'text-yellow-500';
      default: return 'text-muted-foreground';
    }
  }
</script>

<Card.Root>
  <Card.Content class="flex items-center justify-between p-4">
    <div class="flex items-center gap-4">
      <Layers class="h-8 w-8 text-muted-foreground" />
      <a href="/dashboard/stacks/{stack.id}" class="hover:underline">
        <p class="font-medium">{stack.name}</p>
        <p class="text-xs {getStatusColor(stack.status)} capitalize">{stack.status}</p>
        <p class="text-xs text-muted-foreground">
          Created {new Date(stack.created_at).toLocaleDateString()}
        </p>
      </a>
    </div>
    <div class="flex items-center gap-2">
      {#if !isViewer}
        {#if stack.status !== 'running'}
          <Button variant="outline" size="icon" onclick={() => ctrl.startStack(stack.id)} disabled={ctrl.actionLoading === stack.id}>
            <Play class="h-4 w-4" />
          </Button>
        {:else}
          <Button variant="outline" size="icon" onclick={() => ctrl.stopStack(stack.id)} disabled={ctrl.actionLoading === stack.id}>
            <Square class="h-4 w-4" />
          </Button>
        {/if}
        <Button variant="outline" size="icon" onclick={() => ctrl.removeStack(stack.id)} disabled={ctrl.actionLoading === stack.id}>
          <Trash2 class="h-4 w-4 text-destructive" />
        </Button>
      {:else}
        <span class="text-xs text-muted-foreground italic px-2">Read Only</span>
      {/if}
    </div>
  </Card.Content>
</Card.Root>
