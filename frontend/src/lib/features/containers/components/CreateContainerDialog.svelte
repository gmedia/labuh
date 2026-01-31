<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import { X, Plus, Eye, EyeOff } from '@lucide/svelte';
  import type { ContainerListController } from '../container-list-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: ContainerListController }>();
</script>

{#if ctrl.showCreateDialog}
<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto py-8">
  <Card.Root class="w-full max-w-2xl mx-4">
    <Card.Header>
      <div class="flex items-center justify-between">
        <Card.Title>Create Container</Card.Title>
        <Button variant="ghost" size="icon" onclick={() => ctrl.showCreateDialog = false}>
          <X class="h-4 w-4" />
        </Button>
      </div>
    </Card.Header>
    <Card.Content class="space-y-6 max-h-[60vh] overflow-y-auto pr-2">
      <!-- Basic Info -->
      <div class="grid gap-4 md:grid-cols-2">
        <div class="space-y-2">
          <Label for="name">Container Name</Label>
          <Input id="name" placeholder="my-container" bind:value={ctrl.newContainer.name} />
        </div>
        <div class="space-y-2">
          <Label for="image">Image</Label>
          <div class="flex gap-2">
            <Input id="image" placeholder="nginx:latest" bind:value={ctrl.newContainer.image} />
            <Button variant="outline" size="sm" onclick={() => ctrl.loadImagePorts()} disabled={!ctrl.newContainer.image}>
              Detect Ports
            </Button>
          </div>
        </div>
      </div>

      <!-- Environment Variables -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <Label>Environment Variables</Label>
          <div class="flex gap-2">
            <Button variant="outline" size="sm" onclick={() => ctrl.showEnvImport = !ctrl.showEnvImport}>
              Import .env
            </Button>
            <Button variant="outline" size="sm" onclick={() => ctrl.addEnvVar()}>
              <Plus class="h-3 w-3 mr-1" /> Add
            </Button>
          </div>
        </div>

        {#if ctrl.showEnvImport}
          <div class="space-y-2 p-3 border rounded-lg bg-muted/50">
            <Label for="envImport">Paste .env content:</Label>
            <Textarea
              id="envImport"
              placeholder="KEY=value\nANOTHER_KEY=another_value"
              bind:value={ctrl.envImportText}
              rows={4}
            />
            <div class="flex gap-2">
              <Button size="sm" onclick={() => ctrl.importEnvFromText()}>Import</Button>
              <Button variant="outline" size="sm" onclick={() => ctrl.showEnvImport = false}>Cancel</Button>
            </div>
          </div>
        {/if}

        {#if ctrl.newContainer.envVars.length > 0}
          <div class="space-y-2">
            {#each ctrl.newContainer.envVars as envVar, index}
              <div class="flex gap-2 items-center">
                <Input
                  placeholder="KEY"
                  bind:value={envVar.key}
                  class="w-1/3"
                />
                <div class="relative flex-1">
                  <Input
                    placeholder="value"
                    bind:value={envVar.value}
                    type={envVar.masked ? 'password' : 'text'}
                  />
                </div>
                <Button variant="ghost" size="icon" onclick={() => envVar.masked = !envVar.masked}>
                  {#if envVar.masked}
                    <EyeOff class="h-4 w-4" />
                  {:else}
                    <Eye class="h-4 w-4" />
                  {/if}
                </Button>
                <Button variant="ghost" size="icon" onclick={() => ctrl.removeEnvVar(index)}>
                  <X class="h-4 w-4 text-destructive" />
                </Button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Port Mappings -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <Label>Port Mappings</Label>
          <Button variant="outline" size="sm" onclick={() => ctrl.addPort()}>
            <Plus class="h-3 w-3 mr-1" /> Add Port
          </Button>
        </div>

        {#if ctrl.newContainer.ports.length > 0}
          <div class="space-y-2">
            {#each ctrl.newContainer.ports as port, index}
              <div class="flex gap-2 items-center">
                <Input
                  placeholder="Host Port (8080)"
                  bind:value={port.hostPort}
                  class="w-1/3"
                />
                <span class="text-muted-foreground">:</span>
                <Input
                  placeholder="Container Port (80)"
                  bind:value={port.containerPort}
                  class="w-1/3"
                />
                <Button variant="ghost" size="icon" onclick={() => ctrl.removePort(index)}>
                  <X class="h-4 w-4 text-destructive" />
                </Button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </Card.Content>
    <Card.Footer class="flex justify-end gap-2">
      <Button variant="outline" onclick={() => ctrl.showCreateDialog = false}>Cancel</Button>
      <Button onclick={() => ctrl.createContainer()} disabled={ctrl.creating || !ctrl.newContainer.name || !ctrl.newContainer.image}>
        {ctrl.creating ? 'Creating...' : 'Create'}
      </Button>
    </Card.Footer>
  </Card.Root>
</div>
{/if}
