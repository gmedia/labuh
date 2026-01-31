<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Settings } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  function getLimitValue(serviceName: string) {
    const limit = ctrl.resourceLimits.find((l: any) => l.service_name === serviceName);
    return {
      cpu: limit?.cpu_limit,
      memory: limit?.memory_limit ? limit.memory_limit / (1024 * 1024) : undefined
    };
  }

  function setCpu(serviceName: string, value: number | undefined) {
    let l = ctrl.resourceLimits.find((l: any) => l.service_name === serviceName);
    if (l) l.cpu_limit = value;
    else ctrl.resourceLimits.push({ id: '', stack_id: ctrl.id, service_name: serviceName, cpu_limit: value, memory_limit: undefined, created_at: '', updated_at: '' });
  }

  function setMemory(serviceName: string, value: number | undefined) {
    let l = ctrl.resourceLimits.find((l: any) => l.service_name === serviceName);
    const bytes = value ? value * 1024 * 1024 : undefined;
    if (l) l.memory_limit = bytes;
    else ctrl.resourceLimits.push({ id: '', stack_id: ctrl.id, service_name: serviceName, cpu_limit: undefined, memory_limit: bytes, created_at: '', updated_at: '' });
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Settings class="h-5 w-5" />
      Resource Limits
    </Card.Title>
    <Card.Description>Configure CPU and Memory constraints per service</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    {#each ctrl.containers as container}
      {@const serviceName = container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}
      <div class="p-4 border rounded-lg space-y-3">
        <div class="flex items-center justify-between">
          <h4 class="font-medium text-sm">{serviceName}</h4>
          <Button
            size="sm"
            variant="outline"
            onclick={() => {
              const vals = getLimitValue(serviceName);
              ctrl.updateResourceLimit(serviceName, vals.cpu, vals.memory);
            }}
            disabled={ctrl.savingResources.has(serviceName) || isViewer}
          >
            {ctrl.savingResources.has(serviceName) ? 'Saving...' : 'Save'}
          </Button>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-1">
            <Label class="text-[10px] uppercase font-bold text-muted-foreground">CPU Limit (Cores)</Label>
            <Input
              type="number"
              step="0.1"
              placeholder="e.g. 0.5"
              bind:value={() => getLimitValue(serviceName).cpu, (v) => setCpu(serviceName, v)}
            />
          </div>
          <div class="space-y-1">
            <Label class="text-[10px] uppercase font-bold text-muted-foreground">Memory Limit (MB)</Label>
            <Input
              type="number"
              placeholder="e.g. 512"
              bind:value={() => getLimitValue(serviceName).memory, (v) => setMemory(serviceName, v)}
            />
          </div>
        </div>
      </div>
    {/each}
  </Card.Content>
</Card.Root>
