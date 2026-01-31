<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Container as ContainerIcon, Image as ImageIcon, Layers, Activity } from '@lucide/svelte';
  import type { DashboardController } from '../dashboard-controller.svelte';

  let { ctrl } = $props<{ ctrl: DashboardController }>();
</script>

<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
  <Card.Root>
    <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
      <Card.Title class="text-sm font-medium">Running Containers</Card.Title>
      <ContainerIcon class="h-4 w-4 text-green-500" />
    </Card.Header>
    <Card.Content>
      <div class="text-2xl font-bold">{ctrl.runningContainers}</div>
      <p class="text-xs text-muted-foreground">{ctrl.containers.length} total</p>
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
      <Card.Title class="text-sm font-medium">Images</Card.Title>
      <ImageIcon class="h-4 w-4 text-blue-500" />
    </Card.Header>
    <Card.Content>
      <div class="text-2xl font-bold">{ctrl.images.length}</div>
      <p class="text-xs text-muted-foreground">local images</p>
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
      <Card.Title class="text-sm font-medium">Stacks</Card.Title>
      <Layers class="h-4 w-4 text-purple-500" />
    </Card.Header>
    <Card.Content>
      <div class="text-2xl font-bold">{ctrl.stacks.length}</div>
      <p class="text-xs text-muted-foreground">{ctrl.runningStacks} running</p>
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
      <Card.Title class="text-sm font-medium">System Status</Card.Title>
      <Activity class="h-4 w-4 text-orange-500" />
    </Card.Header>
    <Card.Content>
      <div class="text-2xl font-bold capitalize">{ctrl.systemHealth?.status || 'unknown'}</div>
      <p class="text-xs text-muted-foreground">v{ctrl.systemHealth?.version || '?'}</p>
    </Card.Content>
  </Card.Root>
</div>
