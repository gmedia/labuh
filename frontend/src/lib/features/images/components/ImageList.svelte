<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Image as ImageIcon, Trash2 } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { ImageController } from '../image-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: ImageController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString();
  }
</script>

{#if ctrl.loading}
  <div class="flex items-center justify-center py-12">
    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
  </div>
{:else if ctrl.images.length === 0}
  <Card.Root>
    <Card.Content class="flex flex-col items-center justify-center py-12 text-center">
      <ImageIcon class="mb-4 h-12 w-12 text-muted-foreground/50" />
      <h3 class="text-lg font-semibold">No images yet</h3>
      <p class="text-sm text-muted-foreground">
        Pull an image from a registry to get started
      </p>
    </Card.Content>
  </Card.Root>
{:else}
  <Card.Root>
    <Card.Header>
      <Card.Title>Local Images</Card.Title>
    </Card.Header>
    <Card.Content>
      <div class="space-y-3">
        {#each ctrl.images as image}
          <div class="flex items-center justify-between rounded-lg border p-3">
            <div class="flex items-center gap-3">
              <ImageIcon class="h-6 w-6 text-muted-foreground" />
              <div>
                <p class="font-medium">{image.repo_tags[0] || image.id.slice(7, 19)}</p>
                <p class="text-xs text-muted-foreground">
                  {formatSize(image.size)} · Created {formatDate(image.created)}
                </p>
              </div>
            </div>
            {#if !isViewer}
              <Button
                variant="outline"
                size="icon"
                onclick={() => ctrl.removeImage(image.id)}
                disabled={ctrl.actionLoading === image.id}
              >
                <Trash2 class="h-4 w-4 text-destructive" />
              </Button>
            {/if}
          </div>
        {/each}
      </div>
    </Card.Content>
  </Card.Root>
{/if}
