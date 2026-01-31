<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import { X } from '@lucide/svelte';
  import type { TemplateController } from '../template-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: TemplateController }>();

  const sampleTemplateJson = JSON.stringify({
    id: "custom-app",
    name: "Custom App",
    description: "My own custom application template",
    icon: "layout-grid",
    compose_content: "version: '3.8'\nservices:\n  app:\n    image: my-app:latest\n    ports:\n      - \"80:80\"",
    default_env: []
  }, null, 2);
</script>

{#if ctrl.showAddDialog}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
  <Card.Root class="w-full max-w-2xl">
    <Card.Header class="flex flex-row items-center justify-between">
      <div>
        <Card.Title>Add New Template</Card.Title>
        <Card.Description>Import templates from a JSON URL or paste JSON content</Card.Description>
      </div>
      <Button variant="ghost" size="icon" onclick={() => ctrl.showAddDialog = false}>
        <X class="h-4 w-4" />
      </Button>
    </Card.Header>
    <Card.Content class="space-y-4">
      <div class="flex p-1 bg-muted rounded-lg w-fit">
        <button
          class="px-4 py-1.5 text-sm font-medium rounded-md transition-all {ctrl.importMode === 'url' ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
          onclick={() => ctrl.importMode = 'url'}
        >
          URL
        </button>
        <button
          class="px-4 py-1.5 text-sm font-medium rounded-md transition-all {ctrl.importMode === 'json' ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
          onclick={() => ctrl.importMode = 'json'}
        >
          JSON Content
        </button>
      </div>

      {#if ctrl.importMode === 'url'}
        <div class="space-y-2">
          <Label for="url">Template URL</Label>
          <Input
            id="url"
            placeholder="https://example.com/template.json"
            bind:value={ctrl.urlInput}
          />
          <p class="text-xs text-muted-foreground">
            The URL should return a JSON object matching the Labuh template format.
          </p>
        </div>
      {:else}
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <Label for="json">JSON Content</Label>
            <Button
              variant="ghost"
              size="sm"
              onclick={() => ctrl.jsonInput = sampleTemplateJson}
            >
              Load Sample
            </Button>
          </div>
          <Textarea
            id="json"
            placeholder={sampleTemplateJson}
            bind:value={ctrl.jsonInput}
            rows={12}
            class="font-mono text-sm"
          />
        </div>
      {/if}
    </Card.Content>
    <Card.Footer class="justify-end gap-2 border-t pt-4">
      <Button variant="outline" onclick={() => ctrl.showAddDialog = false}>Cancel</Button>
      <Button onclick={() => ctrl.addTemplate()} disabled={ctrl.adding || (ctrl.importMode === 'url' ? !ctrl.urlInput : !ctrl.jsonInput)}>
        {ctrl.adding ? 'Adding...' : 'Add Template'}
      </Button>
    </Card.Footer>
  </Card.Root>
</div>
{/if}
