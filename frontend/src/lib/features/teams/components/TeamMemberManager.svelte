<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Shield, UserPlus, Star, Trash2, Users } from '@lucide/svelte';
  import { activeTeam, auth } from '$lib/stores';
  import ConfirmationDialog from '$lib/components/ConfirmationDialog.svelte';
  import type { TeamController } from '../team-controller.svelte';
  import type { TeamRole, TeamMember } from '$lib/api';

  let { ctrl = $bindable() } = $props<{ ctrl: TeamController }>();

  $effect(() => {
    if ($activeTeam?.team) {
      ctrl.loadMembers($activeTeam.team.id);
    }
  });

  const rolePriority = (role: TeamRole): number => {
    switch (role) {
      case 'Owner': return 4;
      case 'Admin': return 3;
      case 'Developer': return 2;
      case 'Viewer': return 1;
      default: return 0;
    }
  };

  const canModifyMember = (member: TeamMember): boolean => {
    if (!$activeTeam?.role) return false;
    const myRole = $activeTeam.role;
    const targetRole = member.role as TeamRole;

    // Cannot modify yourself
    if ($auth.user?.id === member.user_id) return false;

    // Cannot modify Owner
    if (targetRole === 'Owner') return false;

    // Owner can modify everyone except other owners
    if (myRole === 'Owner') return true;

    // Can only modify members with lower priority role
    return rolePriority(myRole) > rolePriority(targetRole);
  };

  const getAssignableRoles = (): TeamRole[] => {
    if (!$activeTeam?.role) return [];
    const myRole = $activeTeam.role;

    // Owner can assign any role except Owner
    if (myRole === 'Owner') {
      return ['Admin', 'Developer', 'Viewer'];
    }

    // Others can only assign roles below their own
    const myPriority = rolePriority(myRole);
    return ctrl.roles.filter((r: TeamRole) => r !== 'Owner' && rolePriority(r) < myPriority);
  };
</script>

<Card.Root>
  {#if $activeTeam?.team}
    <Card.Header>
      <div class="flex items-center justify-between">
        <div>
          <Card.Title>{$activeTeam?.team?.name || 'Selected Team'}</Card.Title>
          <Card.Description>Manage members and roles for this team</Card.Description>
        </div>
        <div class="flex items-center gap-2 px-3 py-1 bg-muted rounded-full">
          <Shield class="h-3.5 w-3.5 text-primary" />
          <span class="text-xs font-semibold">Your Role: {$activeTeam.role}</span>
        </div>
      </div>
    </Card.Header>
    <Card.Content class="space-y-6">
      <!-- Invite Section -->
      {#if $activeTeam?.role === 'Owner' || $activeTeam?.role === 'Admin'}
        <div class="grid gap-4 p-4 border rounded-lg bg-muted/30">
          <div class="flex items-center gap-2 text-sm font-medium">
            <UserPlus class="h-4 w-4" />
            Invite Member
          </div>
          <div class="flex flex-col gap-3">
            <div class="flex flex-wrap gap-3">
              <div class="flex-1 min-w-[200px]">
                <Input placeholder="Full Name" bind:value={ctrl.inviteName} />
              </div>
              <div class="flex-1 min-w-[200px]">
                <Input type="email" placeholder="member@example.com" bind:value={ctrl.inviteEmail} />
              </div>
            </div>
            <div class="flex flex-wrap gap-3">
              <div class="flex-1 min-w-[200px]">
                <Input type="password" placeholder="Initial Password" bind:value={ctrl.invitePassword} />
              </div>
              <div class="w-32">
                <select
                  bind:value={ctrl.inviteRole}
                  class="w-full flex h-10 items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  {#each ctrl.roles as r}
                    <option value={r}>{r}</option>
                  {/each}
                </select>
              </div>
              <Button onclick={() => ctrl.inviteMember($activeTeam.team!.id)} disabled={ctrl.inviting || !ctrl.inviteEmail || !ctrl.inviteName || !ctrl.invitePassword}>
                {ctrl.inviting ? 'Creating...' : 'Create & Invite'}
              </Button>
            </div>
          </div>
        </div>
      {/if}

      <!-- Members List -->
      <div class="space-y-4">
        <h4 class="font-medium text-sm flex items-center gap-2">
          Team Members
          <span class="text-xs font-normal text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
            {ctrl.selectedTeamMembers.length}
          </span>
        </h4>

        {#if ctrl.loadingMembers}
          <div class="flex justify-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          </div>
        {:else}
          <div class="grid gap-2">
            {#each ctrl.selectedTeamMembers as member}
              <div class="flex items-center justify-between p-3 border rounded-lg bg-card/50">
                <div class="flex items-center gap-3">
                  <div class="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center text-primary font-bold text-xs uppercase">
                    {member.user_name?.slice(0, 2) || member.user_email?.slice(0, 2) || '??'}
                  </div>
                  <div class="grid gap-0.5">
                    <p class="text-sm font-medium leading-none flex items-center gap-1.5">
                      {member.user_name || 'No Name'}
                      {#if member.role === 'Owner'}
                        <Star class="h-3 w-3 fill-yellow-400 text-yellow-400" />
                      {/if}
                    </p>
                    <p class="text-xs text-muted-foreground">
                      {member.user_email}
                    </p>
                  </div>
                </div>

                <div class="flex items-center gap-3">
                  {#if canModifyMember(member)}
                    <select
                      value={member.role}
                      onchange={(e) => ctrl.updateRole($activeTeam.team!.id, member.user_id, e.currentTarget.value as TeamRole)}
                      class="text-xs bg-transparent border-none focus:ring-0 cursor-pointer font-medium hover:text-primary transition-colors"
                    >
                      <option value={member.role}>{member.role}</option>
                      {#each getAssignableRoles() as r}
                        {#if r !== member.role}
                          <option value={r}>{r}</option>
                        {/if}
                      {/each}
                    </select>

                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8 text-destructive hover:bg-destructive/10"
                      onclick={() => ctrl.requestRemoveMember($activeTeam.team!.id, member.user_id)}
                    >
                      <Trash2 class="h-4 w-4" />
                    </Button>
                  {:else}
                    <span class="text-xs font-medium text-muted-foreground px-2 py-1 bg-muted rounded">
                      {member.role}
                    </span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </Card.Content>
  {:else}
    <div class="flex flex-col items-center justify-center p-12 text-center h-[400px]">
      <div class="h-16 w-16 rounded-full bg-muted flex items-center justify-center mb-4">
        <Users class="h-8 w-8 text-muted-foreground" />
      </div>
      <h3 class="text-lg font-medium">No team selected</h3>
      <p class="text-sm text-muted-foreground max-w-sm mt-1">
        Please select a team from the list on the left to manage its members and settings.
      </p>
    </div>
  {/if}
</Card.Root>

<ConfirmationDialog
  bind:open={ctrl.showRemoveMemberConfirm}
  title="Remove Member"
  description="Are you sure you want to remove this member from the team?"
  confirmText="Remove"
  variant="destructive"
  onConfirm={() => ctrl.confirmRemoveMember()}
/>
