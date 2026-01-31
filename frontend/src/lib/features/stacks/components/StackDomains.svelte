<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Globe, CheckCircle, AlertCircle, Trash2 } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  let newDomain = $state('');
  let newDomainContainer = $state('');
  let newDomainPort = $state(80);

  async function handleAdd() {
    if (!newDomain || !newDomainContainer) return;
    await ctrl.addDomain({
      domain: newDomain,
      container_name: newDomainContainer,
      container_port: newDomainPort
    });
    newDomain = '';
    newDomainContainer = '';
    newDomainPort = 80;
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Globe class="h-5 w-5" />
      Domains
    </Card.Title>
    <Card.Description>Manage custom domains</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    <div class="grid gap-2">
      <div class="flex gap-2">
        <Input placeholder="example.com" bind:value={newDomain} class="flex-1" />
      </div>
      <div class="flex gap-2">
        <select
          bind:value={newDomainContainer}
          class="flex-1 h-9 rounded-md border border-input bg-background px-3 text-sm"
        >
          <option value="">Select container...</option>
          {#each ctrl.containers as container}
            <option value={container.names[0]?.replace(/^\//, '') || container.id}>
              {container.names[0]?.replace(/^\//, '') || container.id.substring(0, 12)}
            </option>
          {/each}
        </select>
        <Input
          type="number"
          placeholder="Port"
          bind:value={newDomainPort}
          class="w-20"
        />
        <Button size="icon" onclick={handleAdd} disabled={ctrl.addingDomain || !newDomain || !newDomainContainer || isViewer}>
          {#if ctrl.addingDomain}
            <div class="animate-spin rounded-full h-3 w-3 border-b-2 border-primary-foreground"></div>
          {:else}
            <div class="h-4 w-4">+</div>
          {/if}
        </Button>
      </div>
    </div>

    <div class="space-y-2">
      {#if ctrl.domains.length === 0}
        <p class="text-xs text-muted-foreground text-center py-2">No domains configured</p>
      {:else}
        {#each ctrl.domains as domain}
          <div class="flex items-center justify-between p-2 rounded border bg-background/50">
            <div class="flex items-center gap-2 overflow-hidden">
              {#if domain.verified}
                <CheckCircle class="h-3 w-3 text-green-500 flex-shrink-0" />
              {:else}
                <AlertCircle class="h-3 w-3 text-yellow-500 flex-shrink-0" />
              {/if}
              <span class="text-sm truncate" title={domain.domain}>{domain.domain}</span>
              <span class="text-xs text-muted-foreground">→ {domain.container_name}:{domain.container_port}</span>
            </div>
            <div class="flex gap-1">
              {#if !isViewer}
                {#if !domain.verified}
                  <Button variant="ghost" size="icon" class="h-6 w-6" onclick={() => ctrl.verifyDomain(domain.domain)} title="Verify">
                    <CheckCircle class="h-3 w-3" />
                  </Button>
                {/if}
                <Button variant="ghost" size="icon" class="h-6 w-6 text-destructive" onclick={() => ctrl.removeDomain(domain.domain)} title="Remove">
                  <Trash2 class="h-3 w-3" />
                </Button>
              {:else}
                <span class="text-[10px] text-muted-foreground mr-1">View Only</span>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </Card.Content>
</Card.Root>
