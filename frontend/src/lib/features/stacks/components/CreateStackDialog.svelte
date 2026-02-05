<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import { FileCode, GitBranch, X } from '@lucide/svelte';
  import type { StackListController } from '../stack-list-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackListController }>();

  const sampleCompose = `version: '3.8'
services:
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
  redis:
    image: redis:alpine`;
</script>

{#if ctrl.showCreateDialog}
<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto py-8">
  <Card.Root class="w-full max-w-2xl mx-4">
    <Card.Header>
      <div class="flex items-center justify-between">
        <Card.Title class="flex items-center gap-2">
          <FileCode class="h-5 w-5" />
          Import Docker Compose
        </Card.Title>
        <Button variant="ghost" size="icon" onclick={() => ctrl.showCreateDialog = false}>
          <X class="h-4 w-4" />
        </Button>
      </div>
      <Card.Description>
        Paste your docker-compose.yml content to create a new stack.
      </Card.Description>
    </Card.Header>
    <Card.Content class="space-y-4">
      <div class="space-y-2">
        <Label for="stackName">Stack Name</Label>
        <Input id="stackName" placeholder="my-stack" bind:value={ctrl.newStack.name} />
      </div>
      <div class="space-y-4">
        <div class="flex p-1 bg-muted rounded-lg">
          <button
            class="flex-1 flex items-center justify-center gap-2 py-1.5 text-sm font-medium rounded-md transition-all {ctrl.importMode === 'manual' ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
            onclick={() => ctrl.importMode = 'manual'}
          >
            <FileCode class="h-4 w-4" />
            Manual
          </button>
          <button
            class="flex-1 flex items-center justify-center gap-2 py-1.5 text-sm font-medium rounded-md transition-all {ctrl.importMode === 'git' ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
            onclick={() => ctrl.importMode = 'git'}
          >
            <GitBranch class="h-4 w-4" />
            Git
          </button>
        </div>

        {#if ctrl.importMode === 'manual'}
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <Label for="compose">docker-compose.yml</Label>
              <Button
                variant="ghost"
                size="sm"
                onclick={() => ctrl.newStack.composeContent = sampleCompose}
              >
                Load Example
              </Button>
            </div>
            <Textarea
              id="compose"
              placeholder={sampleCompose}
              bind:value={ctrl.newStack.composeContent}
              rows={10}
              class="font-mono text-sm"
            />
          </div>
        {:else}
          <div class="space-y-4">
            <div class="space-y-2">
              <Label for="gitUrl">Repository URL</Label>
              <Input id="gitUrl" placeholder="https://github.com/user/repo" bind:value={ctrl.gitStack.url} />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-2">
                <Label for="gitBranch">Branch</Label>
                <Input id="gitBranch" placeholder="main" bind:value={ctrl.gitStack.branch} />
              </div>
              <div class="space-y-2">
                <Label for="composePath">Compose Path</Label>
                <Input id="composePath" placeholder="docker-compose.yml" bind:value={ctrl.gitStack.composePath} />
              </div>
            </div>
            <div class="space-y-2">
              <Label for="envContent">Environment Variables <span class="text-muted-foreground text-xs font-normal">(optional)</span></Label>
              <Textarea
                id="envContent"
                placeholder={"# Paste your .env file content here\nDB_HOST=localhost\nDB_PORT=3306\nAPP_KEY=your-secret-key"}
                bind:value={ctrl.gitStack.envContent}
                rows={5}
                class="font-mono text-sm"
              />
              <p class="text-xs text-muted-foreground">Paste your <code>.env</code> file content. Lines starting with # are ignored.</p>
            </div>
          </div>
        {/if}
      </div>
      <div class="text-sm text-muted-foreground border-l-2 border-blue-500 pl-3 py-1 bg-blue-500/5">
        <p><strong>Pro Tip:</strong> Dockerfile builds are supported! If your compose file uses <code>build:</code>, Labuh will handle it automatically.</p>
      </div>
    </Card.Content>
    <Card.Footer class="flex justify-end gap-2">
      <Button variant="outline" onclick={() => ctrl.showCreateDialog = false}>Cancel</Button>
      <Button
        onclick={() => ctrl.createStack()}
        disabled={ctrl.creating || !ctrl.newStack.name || (ctrl.importMode === 'manual' ? !ctrl.newStack.composeContent : !ctrl.gitStack.url)}
      >
        {ctrl.creating ? 'Creating...' : 'Create Stack'}
      </Button>
    </Card.Footer>
  </Card.Root>
</div>
{/if}
