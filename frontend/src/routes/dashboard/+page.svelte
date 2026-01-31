<script lang="ts">
	import { onMount } from 'svelte';
	import { activeTeam } from '$lib/stores';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Container as ContainerIcon, Image as ImageIcon, Layers, Users } from '@lucide/svelte';
	import { DashboardController } from '$lib/features/dashboard/dashboard-controller.svelte';
	import StatCards from '$lib/features/dashboard/components/StatCards.svelte';
	import SystemOverview from '$lib/features/dashboard/components/SystemOverview.svelte';

	let ctrl = $state(new DashboardController());

	onMount(() => {
		ctrl.init();
	});

	$effect(() => {
		if ($activeTeam) {
			ctrl.loadAll();
		}
	});
</script>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Dashboard</h2>
		<p class="text-muted-foreground">Welcome to Labuh - Your lightweight PaaS platform</p>
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
				<p class="mb-4 text-sm text-muted-foreground">
					Please select or create a team to view your dashboard.
				</p>
				<Button href="/dashboard/teams">Go to Teams</Button>
			</Card.Content>
		</Card.Root>
	{:else}
		<StatCards {ctrl} />
		<SystemOverview {ctrl} />

		<!-- Quick Actions -->
		<Card.Root>
			<Card.Header>
				<Card.Title>Quick Actions</Card.Title>
				<Card.Description>Common tasks to get started</Card.Description>
			</Card.Header>
			<Card.Content>
				<div class="grid gap-2 sm:grid-cols-3">
					<a href="/dashboard/containers" class="flex items-center gap-2 rounded-lg border p-3 hover:bg-accent transition-colors">
						<ContainerIcon class="h-5 w-5 text-muted-foreground" />
						<span>View Containers</span>
					</a>
					<a href="/dashboard/images" class="flex items-center gap-2 rounded-lg border p-3 hover:bg-accent transition-colors">
						<ImageIcon class="h-5 w-5 text-muted-foreground" />
						<span>Pull Image</span>
					</a>
					<a href="/dashboard/stacks" class="flex items-center gap-2 rounded-lg border p-3 hover:bg-accent transition-colors">
						<Layers class="h-5 w-5 text-muted-foreground" />
						<span>Create Stack</span>
					</a>
				</div>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
