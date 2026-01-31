<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { HeartPulse, CheckCircle2, AlertCircle, Timer } from '@lucide/svelte';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();

  function getHealthIcon(status: string) {
    if (status.includes('healthy') && !status.includes('unhealthy')) return CheckCircle2;
    if (status.includes('unhealthy')) return AlertCircle;
    if (status.includes('starting')) return Timer;
    return HeartPulse;
  }

  function getHealthColor(status: string) {
    if (status.includes('healthy') && !status.includes('unhealthy')) return 'text-green-500';
    if (status.includes('unhealthy')) return 'text-red-500';
    if (status.includes('starting')) return 'text-yellow-500';
    return 'text-muted-foreground';
  }
</script>

<Card.Root>
  <Card.Header class="pb-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <HeartPulse class="h-5 w-5 text-primary" />
        <Card.Title class="text-lg">Stack Health</Card.Title>
      </div>
      {#if ctrl.health}
        <div class="text-sm font-medium">
          <span class="text-green-500">{ctrl.health.healthy_count}</span>
          <span class="text-muted-foreground">/</span>
          <span>{ctrl.health.total_count} Healthy</span>
        </div>
      {/if}
    </div>
  </Card.Header>
  <Card.Content>
    {#if !ctrl.health}
      <div class="flex items-center justify-center py-4 text-muted-foreground text-sm italic">
        Loading health data...
      </div>
    {:else if ctrl.health.containers.length === 0}
      <div class="text-center py-4 text-muted-foreground text-sm">
        No containers found in this stack.
      </div>
    {:else}
      <div class="space-y-3">
        {#each ctrl.health.containers as container}
          {@const HealthIcon = getHealthIcon(container.status)}
          <div class="flex items-center justify-between p-2 rounded-lg border bg-muted/30">
            <div class="flex items-center gap-3 overflow-hidden">
              <HealthIcon
                class="h-4 w-4 flex-shrink-0 {getHealthColor(container.status)}"
              />
              <span class="text-sm font-medium truncate">{container.name}</span>
            </div>
            <div class="flex items-center gap-2 ml-2 flex-shrink-0">
              <span class="text-[10px] px-1.5 py-0.5 rounded-full border bg-background font-mono capitalize">
                {container.state}
              </span>
              <span class="text-[10px] font-medium uppercase {getHealthColor(container.status)}">
                {container.status}
              </span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </Card.Content>
</Card.Root>
