<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Settings, Plus, Eye, EyeOff, Trash2 } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  let newEnvKey = $state('');
  let newEnvValue = $state('');
  let newEnvContainer = $state('');
  let newEnvSecret = $state(false);

  async function handleAdd() {
    if (!newEnvKey) return;
    await ctrl.addEnvVar({
      container_name: newEnvContainer,
      key: newEnvKey,
      value: newEnvValue,
      is_secret: newEnvSecret
    });
    newEnvKey = '';
    newEnvValue = '';
    newEnvContainer = '';
    newEnvSecret = false;
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Settings class="h-5 w-5" />
      Environment Variables
    </Card.Title>
    <Card.Description>Configure stack environment</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    <div class="space-y-2">
      <div class="flex gap-2">
        <Input placeholder="KEY" bind:value={newEnvKey} class="flex-1 font-mono text-xs" />
        <Input placeholder="value" bind:value={newEnvValue} class="flex-1 font-mono text-xs" />
      </div>
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <label class="flex items-center gap-2 text-xs cursor-pointer">
            <input type="checkbox" bind:checked={newEnvSecret} class="rounded" />
            Secret
          </label>

          <select
            bind:value={newEnvContainer}
            class="h-8 rounded-md border border-input bg-background px-2 text-[10px]"
          >
            <option value="">Global</option>
            {#each ctrl.containers as container}
              <option value={container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}>
                {container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}
              </option>
            {/each}
          </select>
        </div>
        <Button size="sm" variant="outline" onclick={handleAdd} disabled={isViewer}>
          <Plus class="h-3 w-3 mr-1" /> Add
        </Button>
      </div>
    </div>

    <div class="space-y-1 max-h-60 overflow-y-auto">
      {#if ctrl.envVars.length === 0}
        <p class="text-xs text-muted-foreground text-center py-2">No environment variables</p>
      {:else}
        {#each ctrl.envVars as env}
          <div class="flex items-center justify-between p-2 rounded border bg-background/50 text-xs gap-2">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5 mb-0.5">
                <span class="font-mono font-medium truncate">{env.key}</span>
                {#if env.container_name}
                  <span class="px-1.5 py-0.5 rounded-full bg-primary/10 text-primary text-[9px]">
                    {env.container_name}
                  </span>
                {:else}
                  <span class="px-1.5 py-0.5 rounded-full bg-muted text-muted-foreground text-[9px]">
                    Global
                  </span>
                {/if}
              </div>
              {#if env.is_secret && !ctrl.showSecrets.has(env.id)}
                <span class="text-muted-foreground">********</span>
              {:else}
                <span class="font-mono text-muted-foreground truncate">{env.value}</span>
              {/if}
            </div>
            <div class="flex items-center gap-1 ml-2">
              {#if env.is_secret}
                <Button variant="ghost" size="icon" class="h-5 w-5" onclick={() => ctrl.toggleSecretVisibility(env.id)} title="Toggle visibility">
                  {#if ctrl.showSecrets.has(env.id)}
                    <EyeOff class="h-3 w-3" />
                  {:else}
                    <Eye class="h-3 w-3" />
                  {/if}
                </Button>
              {/if}
              {#if !isViewer}
                <Button variant="ghost" size="icon" class="h-5 w-5 text-destructive" onclick={() => ctrl.deleteEnvVar(env.key, env.container_name)} title="Delete">
                  <Trash2 class="h-3 w-3" />
                </Button>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
    <p class="text-xs text-muted-foreground">
      Changes apply after stack restart.
    </p>
  </Card.Content>
</Card.Root>
