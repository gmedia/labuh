<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Play, Save } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  let cronSchedule = $state('');
  let healthCheckPath = $state('');
  let healthCheckInterval = $state(30);

  $effect(() => {
    if (ctrl.stack) {
      cronSchedule = ctrl.stack.cron_schedule || '';
      healthCheckPath = ctrl.stack.health_check_path || '';
      healthCheckInterval = ctrl.stack.health_check_interval || 30;
    }
  });

  async function handleSave() {
    await ctrl.saveAutomation({
      cron_schedule: cronSchedule,
      health_check_path: healthCheckPath,
      health_check_interval: healthCheckInterval
    });
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Play class="h-5 w-5" />
      Automation
    </Card.Title>
    <Card.Description>Scheduled pulls & health checks</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    <div class="space-y-2">
      <Label class="text-xs">Cron Schedule (e.g. 0 0 * * * *)</Label>
      <Input bind:value={cronSchedule} placeholder="0 0 * * * *" class="font-mono text-xs" />
      <p class="text-[10px] text-muted-foreground italic">Standard cron: sec min hour day month dow</p>
    </div>
    <div class="space-y-2">
      <Label class="text-xs">Health Check Path (HTTP URL)</Label>
      <Input bind:value={healthCheckPath} placeholder="http://yourapp.labuh:8080/health" class="font-mono text-xs" />
    </div>
    <div class="space-y-2">
      <Label class="text-xs">Interval (seconds)</Label>
      <Input type="number" bind:value={healthCheckInterval} class="font-mono text-xs" />
    </div>
    <Button variant="outline" size="sm" class="w-full" onclick={handleSave} disabled={ctrl.savingAutomation || isViewer}>
      <Save class="h-3 w-3 mr-1" /> {ctrl.savingAutomation ? 'Saving...' : 'Save Automation'}
    </Button>
  </Card.Content>
</Card.Root>
