<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Textarea } from '$lib/components/ui/textarea';
  import { FileCode, Save } from '@lucide/svelte';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
</script>

{#if ctrl.showComposeEditor}
<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <Card.Title class="flex items-center gap-2">
        <FileCode class="h-5 w-5" />
        docker-compose.yml
      </Card.Title>
      <div class="flex gap-2">
        <Button variant="outline" size="sm" onclick={() => ctrl.showComposeEditor = false}>Cancel</Button>
        <Button size="sm" onclick={() => ctrl.saveCompose()} disabled={ctrl.savingCompose}>
          <Save class="h-4 w-4 mr-1" />
          {ctrl.savingCompose ? 'Saving...' : 'Save & Redeploy'}
        </Button>
      </div>
    </div>
  </Card.Header>
  <Card.Content>
    <Textarea
      bind:value={ctrl.editedCompose}
      rows={15}
      class="font-mono text-sm"
      placeholder="version: '3.8'..."
    />
    <p class="mt-2 text-xs text-muted-foreground">
      <strong>Warning:</strong> Saving will stop and recreate all containers in this stack.
    </p>
  </Card.Content>
</Card.Root>
{/if}
