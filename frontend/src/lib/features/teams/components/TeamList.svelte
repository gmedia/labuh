<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Users, Plus } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { TeamController } from '../team-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: TeamController }>();
</script>

<Card.Root class="h-fit">
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Users class="h-5 w-5" />
      Your Teams
    </Card.Title>
    <Card.Description>Teams you are a member of</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    <div class="space-y-2">
      <div class="flex gap-2">
        <Input placeholder="Team name" bind:value={ctrl.newTeamName} />
        <Button size="icon" onclick={() => ctrl.createTeam()} disabled={ctrl.creatingTeam || !ctrl.newTeamName}>
          <Plus class="h-4 w-4" />
        </Button>
      </div>
    </div>

    <div class="space-y-1">
      {#if ctrl.loadingTeams}
        <div class="flex justify-center p-4">
          <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-primary"></div>
        </div>
      {:else if ctrl.teams.length === 0}
        <p class="text-xs text-center text-muted-foreground py-4">No teams found.</p>
      {:else}
        {#each ctrl.teams as t}
          <button
            onclick={() => activeTeam.setActiveTeam(t)}
            class="flex w-full items-center justify-between p-2 text-sm rounded-md transition-colors hover:bg-muted {$activeTeam?.team?.id === t.team?.id ? 'bg-primary/10 text-primary font-medium' : ''}"
          >
            <div class="flex items-center gap-2">
              <div class="h-2 w-2 rounded-full bg-primary/40"></div>
              <span>{t.team?.name || 'Unknown Team'}</span>
            </div>
            <span class="text-[10px] uppercase font-bold text-muted-foreground bg-muted-foreground/10 px-1.5 py-0.5 rounded">
              {t.role}
            </span>
          </button>
        {/each}
      {/if}
    </div>
  </Card.Content>
</Card.Root>
