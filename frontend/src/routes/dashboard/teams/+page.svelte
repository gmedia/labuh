<script lang="ts">
	import { onMount } from 'svelte';
	import { TeamController } from '$lib/features/teams/team-controller.svelte';
	import TeamList from '$lib/features/teams/components/TeamList.svelte';
	import TeamMemberManager from '$lib/features/teams/components/TeamMemberManager.svelte';

	let ctrl = $state(new TeamController());

	onMount(() => {
		ctrl.init();
	});
</script>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Teams</h2>
		<p class="text-muted-foreground">Manage your teams and collaborations</p>
	</div>

	<div class="grid gap-6 lg:grid-cols-3">
		<TeamList bind:ctrl />
		<div class="lg:col-span-2">
			<TeamMemberManager bind:ctrl />
		</div>
	</div>
</div>
