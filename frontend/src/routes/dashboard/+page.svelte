<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type SystemStats, type Container, type Image, type Stack } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Container as ContainerIcon, Image as ImageIcon, Layers, Activity, Cpu, HardDrive, Clock } from '@lucide/svelte';

	let systemHealth = $state<{ status: string; version: string } | null>(null);
	let systemStats = $state<SystemStats | null>(null);
	let containers = $state<Container[]>([]);
	let images = $state<Image[]>([]);
	let stacks = $state<Stack[]>([]);
	let loading = $state(true);

	onMount(async () => {
		// Fetch all data
		const [healthRes, statsRes, containersRes, imagesRes, stacksRes] = await Promise.all([
			api.health.check(),
			api.system.stats(),
			api.containers.list(true),
			api.images.list(),
			api.stacks.list(),
		]);

		if (healthRes.data) systemHealth = healthRes.data;
		if (statsRes.data) systemStats = statsRes.data;
		if (containersRes.data) containers = containersRes.data;
		if (imagesRes.data) images = imagesRes.data;
		if (stacksRes.data) stacks = stacksRes.data;

		loading = false;
	});

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	function formatUptime(seconds: number): string {
		const days = Math.floor(seconds / 86400);
		const hours = Math.floor((seconds % 86400) / 3600);
		const mins = Math.floor((seconds % 3600) / 60);
		if (days > 0) return `${days}d ${hours}h`;
		if (hours > 0) return `${hours}h ${mins}m`;
		return `${mins}m`;
	}

	let runningContainers = $derived(containers.filter(c => c.state === 'running').length);
</script>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Dashboard</h2>
		<p class="text-muted-foreground">Welcome to Labuh - Your lightweight PaaS platform</p>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else}
		<!-- Stats Cards -->
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Running Containers</Card.Title>
					<ContainerIcon class="h-4 w-4 text-green-500" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{runningContainers}</div>
					<p class="text-xs text-muted-foreground">{containers.length} total</p>
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Images</Card.Title>
					<ImageIcon class="h-4 w-4 text-blue-500" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{images.length}</div>
					<p class="text-xs text-muted-foreground">local images</p>
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Stacks</Card.Title>
					<Layers class="h-4 w-4 text-purple-500" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{stacks.length}</div>
					<p class="text-xs text-muted-foreground">{stacks.filter(s => s.status === 'running').length} running</p>
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">System Status</Card.Title>
					<Activity class="h-4 w-4 text-orange-500" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold capitalize">{systemHealth?.status || 'unknown'}</div>
					<p class="text-xs text-muted-foreground">v{systemHealth?.version || '?'}</p>
				</Card.Content>
			</Card.Root>
		</div>

		<!-- System Overview -->
		{#if systemStats}
			<Card.Root>
				<Card.Header>
					<Card.Title>System Overview</Card.Title>
					<Card.Description>Server resource usage</Card.Description>
				</Card.Header>
				<Card.Content>
					<div class="grid gap-4 md:grid-cols-4">
						<div class="flex items-center gap-3">
							<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-500/10 text-blue-500">
								<Cpu class="h-5 w-5" />
							</div>
							<div>
								<p class="text-sm text-muted-foreground">CPU Cores</p>
								<p class="text-lg font-semibold">{systemStats.cpu_count}</p>
							</div>
						</div>

						<div class="flex items-center gap-3">
							<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-green-500/10 text-green-500">
								<HardDrive class="h-5 w-5" />
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Memory</p>
								<p class="text-lg font-semibold">{systemStats.memory_used_percent.toFixed(1)}%</p>
								<p class="text-xs text-muted-foreground">
									{formatBytes(systemStats.memory_available_kb * 1024)} free
								</p>
							</div>
						</div>

						<div class="flex items-center gap-3">
							<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-purple-500/10 text-purple-500">
								<HardDrive class="h-5 w-5" />
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Disk</p>
								<p class="text-lg font-semibold">{systemStats.disk_used_percent.toFixed(1)}%</p>
								<p class="text-xs text-muted-foreground">
									{formatBytes(systemStats.disk_available_bytes)} free
								</p>
							</div>
						</div>

						<div class="flex items-center gap-3">
							<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-orange-500/10 text-orange-500">
								<Clock class="h-5 w-5" />
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Uptime</p>
								<p class="text-lg font-semibold">{formatUptime(systemStats.uptime_seconds)}</p>
								<p class="text-xs text-muted-foreground">
									Load: {systemStats.load_average.one.toFixed(2)}
								</p>
							</div>
						</div>
					</div>
				</Card.Content>
			</Card.Root>
		{/if}

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
