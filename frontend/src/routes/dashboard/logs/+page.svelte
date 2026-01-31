<script lang="ts">
	import { onMount } from 'svelte';
	import { activeTeam } from '$lib/stores';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Terminal, Users } from '@lucide/svelte';
	import { LogsController } from '$lib/features/logs/logs-controller.svelte';
	import ContainerSidebar from '$lib/features/logs/components/ContainerSidebar.svelte';
	import LogConsole from '$lib/features/logs/components/LogConsole.svelte';

	let ctrl = $state(new LogsController());

	onMount(() => {
		ctrl.init();
	});

	$effect(() => {
		if ($activeTeam) {
			ctrl.loadContainers();
		}
	});
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Logs</h2>
			<p class="text-muted-foreground">View and search container logs</p>
		</div>
	</div>

	{#if ctrl.loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if !$activeTeam?.team}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<Users class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No team selected</h3>
				<p class="mb-4 text-sm text-muted-foreground">Please select a team to view container logs.</p>
				<Button href="/dashboard/teams">Go to Teams</Button>
			</Card.Content>
		</Card.Root>
	{:else if ctrl.containers.length === 0}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<Terminal class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No containers available</h3>
				<p class="text-sm text-muted-foreground">Create some containers first to view their logs.</p>
			</Card.Content>
		</Card.Root>
	{:else}
		<div class="grid gap-4 lg:grid-cols-4">
			<ContainerSidebar bind:ctrl />
			<LogConsole bind:ctrl />
		</div>
	{/if}
</div>
