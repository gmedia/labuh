<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Cpu, HardDrive, Clock } from '@lucide/svelte';
  import type { DashboardController } from '../dashboard-controller.svelte';

  let { ctrl } = $props<{ ctrl: DashboardController }>();

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${mins}m`;
    return `${mins}m`;
  }
</script>

{#if ctrl.systemStats}
<Card.Root>
  <Card.Header>
    <Card.Title>System Overview</Card.Title>
    <Card.Description>Server resource usage</Card.Description>
  </Card.Header>
  <Card.Content>
    <div class="grid gap-4 md:grid-cols-4">
      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-500/10 text-blue-500">
          <Cpu class="h-5 w-5" />
        </div>
        <div>
          <p class="text-sm text-muted-foreground">CPU Cores</p>
          <p class="text-lg font-semibold">{ctrl.systemStats.cpu_count}</p>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-green-500/10 text-green-500">
          <HardDrive class="h-5 w-5" />
        </div>
        <div>
          <p class="text-sm text-muted-foreground">Memory</p>
          <p class="text-lg font-semibold">{ctrl.systemStats.memory_used_percent.toFixed(1)}%</p>
          <p class="text-xs text-muted-foreground">
            {formatBytes(ctrl.systemStats.memory_available_kb * 1024)} free
          </p>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-purple-500/10 text-purple-500">
          <HardDrive class="h-5 w-5" />
        </div>
        <div>
          <p class="text-sm text-muted-foreground">Disk</p>
          <p class="text-lg font-semibold">{ctrl.systemStats.disk_used_percent.toFixed(1)}%</p>
          <p class="text-xs text-muted-foreground">
            {formatBytes(ctrl.systemStats.disk_available_bytes)} free
          </p>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-orange-500/10 text-orange-500">
          <Clock class="h-5 w-5" />
        </div>
        <div>
          <p class="text-sm text-muted-foreground">Uptime</p>
          <p class="text-lg font-semibold">{formatUptime(ctrl.systemStats.uptime_seconds)}</p>
          <p class="text-xs text-muted-foreground">
            Load: {ctrl.systemStats.load_average.one.toFixed(2)}
          </p>
        </div>
      </div>
    </div>
  </Card.Content>
</Card.Root>
{/if}
