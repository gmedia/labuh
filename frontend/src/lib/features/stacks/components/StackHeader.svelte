<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import {
    ArrowLeft, Play, Square, Trash2, RefreshCw, FileCode,
    RotateCcw, GitBranch, Download, Users, Layers
  } from '@lucide/svelte';
  import { goto } from '$app/navigation';
  import { activeTeam } from '$lib/stores';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  function getStatusColor(state: string | undefined): string {
    switch (state) {
      case 'running': return 'text-green-500';
      case 'exited': return 'text-red-500';
      case 'paused': return 'text-yellow-500';
      default: return 'text-muted-foreground';
    }
  }
</script>

<div class="flex items-center gap-4">
  <Button variant="ghost" size="icon" onclick={() => goto('/dashboard/stacks')}>
    <ArrowLeft class="h-5 w-5" />
  </Button>
  <div class="flex-1">
    <div class="flex items-center gap-2">
      <h2 class="text-2xl font-bold tracking-tight">{ctrl.stack?.name || 'Loading...'}</h2>
      {#if $activeTeam?.team}
        <span class="text-[10px] uppercase font-bold text-muted-foreground bg-muted-foreground/10 px-2 py-0.5 rounded flex items-center gap-1">
          <Users class="h-2.5 w-2.5" />
          {$activeTeam.team.name}
        </span>
      {/if}
    </div>
    <p class="text-muted-foreground">Stack Details</p>
  </div>
  <Button variant="ghost" size="sm" onclick={() => ctrl.loadStack()}>
    <RefreshCw class="h-4 w-4" />
  </Button>
</div>

{#if ctrl.stack}
<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <Layers class="h-6 w-6" />
        <div>
          <Card.Title>{ctrl.stack.name}</Card.Title>
          <p class="text-sm text-muted-foreground capitalize {getStatusColor(ctrl.stack.status)}">{ctrl.stack.status}</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        {#if !isViewer}
          {#if ctrl.stack.status !== 'running'}
            <Button variant="outline" size="sm" onclick={() => ctrl.start()} disabled={ctrl.actionLoading}>
              <Play class="h-4 w-4 mr-1" /> Start
            </Button>
          {:else}
            <Button variant="outline" size="sm" onclick={() => ctrl.stop()} disabled={ctrl.actionLoading}>
              <Square class="h-4 w-4 mr-1" /> Stop
            </Button>
          {/if}
          <Button
            variant="outline"
            size="sm"
            onclick={() => ctrl.rollback()}
            disabled={ctrl.actionLoading || !ctrl.stack?.last_stable_images}
            title="Rollback to last stable version"
          >
            <RotateCcw class="h-4 w-4 mr-1 {ctrl.actionLoading ? 'animate-spin' : ''}" />
            {ctrl.actionLoading ? 'Rolling back...' : 'Rollback'}
          </Button>
          <Button variant="outline" size="sm" onclick={() => ctrl.redeploy()} disabled={ctrl.actionLoading} title="Recreate containers to apply changes">
            <RotateCcw class="h-4 w-4 mr-1" /> Restart
          </Button>
          <Button variant="outline" size="sm" onclick={() => ctrl.showComposeEditor = !ctrl.showComposeEditor}>
            <FileCode class="h-4 w-4 mr-1" /> Edit
          </Button>
          {#if ctrl.stack.git_url}
            <Button variant="outline" size="sm" onclick={() => ctrl.syncGit()} disabled={ctrl.actionLoading} title="Sync with Git and redeploy">
              <GitBranch class="h-4 w-4 mr-1 {ctrl.actionLoading ? 'animate-spin' : ''}" /> Sync
            </Button>
          {/if}
          <Button variant="outline" size="sm" onclick={() => ctrl.export()} disabled={ctrl.actionLoading} title="Export stack as JSON backup">
            <Download class="h-4 w-4 mr-1" /> Export
          </Button>
          <Button variant="outline" size="sm" onclick={() => ctrl.remove()} disabled={ctrl.actionLoading}>
            <Trash2 class="h-4 w-4 text-destructive" />
          </Button>
        {:else}
          <span class="text-xs text-muted-foreground italic px-2">Read Only Mode</span>
        {/if}
      </div>
    </div>
  </Card.Header>
  <Card.Content>
    <div class="grid gap-4 md:grid-cols-4">
      <div>
        <p class="text-sm text-muted-foreground">Created</p>
        <p class="font-medium">{new Date(ctrl.stack.created_at).toLocaleDateString()}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Containers</p>
        <p class="font-medium">{ctrl.containers.length}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Running</p>
        <p class="font-medium text-green-500">{ctrl.runningCount}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Stopped</p>
        <p class="font-medium text-red-500">{ctrl.stoppedCount}</p>
      </div>
    </div>
  </Card.Content>
</Card.Root>
{/if}
