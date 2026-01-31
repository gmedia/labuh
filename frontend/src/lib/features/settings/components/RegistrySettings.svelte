<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Container, Trash2, Plus, Users } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { SettingsController } from '../settings-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: SettingsController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');
</script>

<Card.Root class="lg:col-span-2">
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Container class="h-5 w-5" />
      Container Registries
    </Card.Title>
    <Card.Description>
      Manage credentials for private container registries (Docker Hub, GHCR, etc.)
    </Card.Description>
  </Card.Header>
  <Card.Content class="space-y-6">
    <!-- Add Registry Form -->
    <div class="grid gap-4 p-4 border rounded-lg bg-muted/30">
      <h4 class="font-medium text-sm">Add New Registry</h4>
      <div class="grid gap-4 md:grid-cols-2">
        <div class="space-y-2">
          <Label for="regName">Name (Alias)</Label>
          <Input id="regName" placeholder="My Docker Hub" bind:value={ctrl.newRegistry.name} />
        </div>
        <div class="space-y-2">
          <Label for="regUrl">Registry URL</Label>
          <Input id="regUrl" placeholder="docker.io or ghcr.io" bind:value={ctrl.newRegistry.registry_url} />
        </div>
        <div class="space-y-2">
          <Label for="regUser">Username</Label>
          <Input id="regUser" placeholder="username" bind:value={ctrl.newRegistry.username} />
        </div>
        <div class="space-y-2">
          <Label for="regPass">Password / Token</Label>
          <Input id="regPass" type="password" placeholder="••••••••" bind:value={ctrl.newRegistry.password} />
        </div>
      </div>
      <div class="flex justify-end">
        <Button onclick={() => ctrl.addRegistry()} disabled={ctrl.addingRegistry || !$activeTeam?.team || isViewer}>
          <Plus class="h-4 w-4 mr-2" />
          {ctrl.addingRegistry ? 'Adding...' : 'Add Credential'}
        </Button>
      </div>
    </div>

    <!-- Registry List -->
    <div>
      <h4 class="mb-4 font-medium text-sm">Saved Registries</h4>
      {#if !$activeTeam?.team}
        <div class="flex flex-col items-center justify-center py-8 text-center bg-muted/20 rounded-lg">
          <Users class="mb-2 h-8 w-8 text-muted-foreground/50" />
          <p class="text-xs text-muted-foreground">Select a team to see registries</p>
        </div>
      {:else if ctrl.loadingRegistries}
        <div class="flex items-center justify-center py-8">
          <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
        </div>
      {:else if ctrl.registries.length === 0}
        <p class="text-sm text-muted-foreground text-center py-4">
          No registry credentials saved.
        </p>
      {:else}
        <div class="space-y-2">
          {#each ctrl.registries as reg}
            <div class="flex items-center justify-between p-3 rounded-md border">
              <div class="grid gap-1">
                <p class="font-medium">{reg.name}</p>
                <div class="flex items-center gap-4 text-xs text-muted-foreground">
                  <span>{reg.registry_url}</span>
                  <span>•</span>
                  <span>{reg.username}</span>
                </div>
              </div>
              <div class="flex items-center gap-2">
                {#if !isViewer}
                  <Button
                    variant="ghost"
                    size="icon"
                    onclick={() => ctrl.removeRegistry(reg.id)}
                  >
                    <Trash2 class="h-4 w-4 text-destructive" />
                  </Button>
                {:else}
                  <span class="text-[10px] text-muted-foreground italic px-2">Read Only</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </Card.Content>
</Card.Root>
