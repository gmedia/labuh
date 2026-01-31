<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Download, Users } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { ImageController } from '../image-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: ImageController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>Pull Image</Card.Title>
    <Card.Description>Pull a container image from Docker Hub or a private registry</Card.Description>
  </Card.Header>
  <Card.Content>
    {#if !$activeTeam?.team}
      <div class="flex flex-col items-center justify-center py-6 text-center bg-muted/20 rounded-lg">
        <Users class="mb-2 h-8 w-8 text-muted-foreground/50" />
        <p class="text-sm text-muted-foreground font-medium">No team selected</p>
        <p class="text-xs text-muted-foreground mt-1">Select a team to pull images with appropriate credentials</p>
        <Button href="/dashboard/teams" variant="link" size="sm">Go to Teams</Button>
      </div>
    {:else}
      <form onsubmit={(e) => { e.preventDefault(); ctrl.pullImage(); }} class="flex gap-2">
        <Input
          placeholder="e.g., nginx:latest, ghcr.io/user/image:tag"
          bind:value={ctrl.imageUrl}
          class="flex-1"
          disabled={ctrl.pulling || isViewer}
        />
        <Button type="submit" disabled={ctrl.pulling || !ctrl.imageUrl || isViewer} class="gap-2">
          <Download class="h-4 w-4" />
          {ctrl.pulling ? 'Pulling...' : 'Pull'}
        </Button>
      </form>
      {#if isViewer}
        <p class="mt-2 text-xs text-muted-foreground italic">Pulling images is restricted for Viewers.</p>
      {:else if ctrl.pulling}
        <p class="mt-2 text-sm text-muted-foreground">Pulling image... This may take a while.</p>
      {/if}
    {/if}
  </Card.Content>
</Card.Root>
