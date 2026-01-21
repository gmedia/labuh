<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Container, Image, FolderKanban, Activity } from '@lucide/svelte';

	let systemHealth = $state<{ status: string; version: string } | null>(null);

	onMount(async () => {
		const result = await api.health.check();
		if (result.data) {
			systemHealth = result.data;
		}
	});

	const baseStats = [
		{ label: 'Running Containers', value: '0', icon: Container, color: 'text-green-500' },
		{ label: 'Images', value: '0', icon: Image, color: 'text-blue-500' },
		{ label: 'Projects', value: '0', icon: FolderKanban, color: 'text-purple-500' },
	];
</script>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Dashboard</h2>
		<p class="text-muted-foreground">Welcome to Labuh - Your lightweight PaaS platform</p>
	</div>

	<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
		{#each baseStats as stat}
			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">{stat.label}</Card.Title>
					<stat.icon class="h-4 w-4 {stat.color}" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{stat.value}</div>
				</Card.Content>
			</Card.Root>
		{/each}
		<Card.Root>
			<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
				<Card.Title class="text-sm font-medium">System Status</Card.Title>
				<Activity class="h-4 w-4 text-orange-500" />
			</Card.Header>
			<Card.Content>
				<div class="text-2xl font-bold">{systemHealth?.status || 'checking...'}</div>
			</Card.Content>
		</Card.Root>
	</div>

	{#if systemHealth}
		<Card.Root>
			<Card.Header>
				<Card.Title>System Information</Card.Title>
				<Card.Description>Backend health status</Card.Description>
			</Card.Header>
			<Card.Content>
				<div class="flex gap-4">
					<div>
						<p class="text-sm text-muted-foreground">Status</p>
						<p class="font-medium capitalize">{systemHealth.status}</p>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">Version</p>
						<p class="font-medium">{systemHealth.version}</p>
					</div>
				</div>
			</Card.Content>
		</Card.Root>
	{/if}

	<Card.Root>
		<Card.Header>
			<Card.Title>Quick Actions</Card.Title>
			<Card.Description>Common tasks to get started</Card.Description>
		</Card.Header>
		<Card.Content>
			<div class="grid gap-2 sm:grid-cols-3">
				<a href="/dashboard/containers" class="flex items-center gap-2 rounded-lg border p-3 hover:bg-accent transition-colors">
					<Container class="h-5 w-5 text-muted-foreground" />
					<span>View Containers</span>
				</a>
				<a href="/dashboard/images" class="flex items-center gap-2 rounded-lg border p-3 hover:bg-accent transition-colors">
					<Image class="h-5 w-5 text-muted-foreground" />
					<span>Pull Image</span>
				</a>
				<a href="/dashboard/projects" class="flex items-center gap-2 rounded-lg border p-3 hover:bg-accent transition-colors">
					<FolderKanban class="h-5 w-5 text-muted-foreground" />
					<span>Create Project</span>
				</a>
			</div>
		</Card.Content>
	</Card.Root>
</div>
