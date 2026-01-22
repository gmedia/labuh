<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { api, type Container, type ContainerStats } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { ArrowLeft, Play, Square, RotateCcw, Trash2, RefreshCw, Terminal } from '@lucide/svelte';

	const containerId: string = $page.params.id ?? '';

	let container = $state<Container | null>(null);
	let stats = $state<ContainerStats | null>(null);
	let logs = $state<string[]>([]);
	let loading = $state(true);
	let logsLoading = $state(false);
	let actionLoading = $state(false);
	let autoRefresh = $state(true);
	let refreshInterval: ReturnType<typeof setInterval> | null = null;

	async function loadContainer() {
		const result = await api.containers.list(true);
		if (result.data && containerId) {
			container = result.data.find(c => c.id === containerId || c.id.startsWith(containerId)) || null;
		}
		loading = false;
	}

	async function loadStats() {
		if (!container) return;
		const result = await api.containers.stats(container.id);
		if (result.data) {
			stats = result.data;
		}
	}

	async function loadLogs() {
		if (!container) return;
		logsLoading = true;
		const result = await api.containers.logs(container.id, 200);
		if (result.data) {
			logs = result.data;
		}
		logsLoading = false;
	}

	onMount(async () => {
		await loadContainer();
		if (container) {
			await Promise.all([loadStats(), loadLogs()]);

			// Auto-refresh stats every 5 seconds
			refreshInterval = setInterval(() => {
				if (autoRefresh && container?.state === 'running') {
					loadStats();
				}
			}, 5000);
		}
	});

	onDestroy(() => {
		if (refreshInterval) {
			clearInterval(refreshInterval);
		}
	});

	async function startContainer() {
		if (!container) return;
		actionLoading = true;
		await api.containers.start(container.id);
		await loadContainer();
		actionLoading = false;
	}

	async function stopContainer() {
		if (!container) return;
		actionLoading = true;
		await api.containers.stop(container.id);
		await loadContainer();
		actionLoading = false;
	}

	async function restartContainer() {
		if (!container) return;
		actionLoading = true;
		await api.containers.restart(container.id);
		await loadContainer();
		actionLoading = false;
	}

	async function removeContainer() {
		if (!container) return;
		if (!confirm('Are you sure you want to delete this container?')) return;
		actionLoading = true;
		await api.containers.remove(container.id);
		goto('/dashboard/containers');
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	function getStatusColor(state: string): string {
		switch (state) {
			case 'running': return 'bg-green-500';
			case 'exited': return 'bg-red-500';
			case 'paused': return 'bg-yellow-500';
			default: return 'bg-muted-foreground';
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" onclick={() => goto('/dashboard/containers')}>
			<ArrowLeft class="h-5 w-5" />
		</Button>
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Container Details</h2>
			<p class="text-muted-foreground">{containerId.slice(0, 12)}</p>
		</div>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if !container}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<h3 class="text-lg font-semibold">Container not found</h3>
				<p class="mb-4 text-sm text-muted-foreground">The container may have been removed.</p>
				<Button onclick={() => goto('/dashboard/containers')}>Back to Containers</Button>
			</Card.Content>
		</Card.Root>
	{:else}
		<!-- Container Info -->
		<Card.Root>
			<Card.Header>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3">
						<span class="h-3 w-3 rounded-full {getStatusColor(container.state)}"></span>
						<Card.Title>{container.names[0]?.replace(/^\//, '') || container.id.slice(0, 12)}</Card.Title>
					</div>
					<div class="flex items-center gap-2">
						{#if container.state !== 'running'}
							<Button variant="outline" size="sm" onclick={startContainer} disabled={actionLoading}>
								<Play class="h-4 w-4 mr-1" /> Start
							</Button>
						{:else}
							<Button variant="outline" size="sm" onclick={stopContainer} disabled={actionLoading}>
								<Square class="h-4 w-4 mr-1" /> Stop
							</Button>
						{/if}
						<Button variant="outline" size="sm" onclick={restartContainer} disabled={actionLoading}>
							<RotateCcw class="h-4 w-4 mr-1" /> Restart
						</Button>
						<Button variant="outline" size="sm" onclick={removeContainer} disabled={actionLoading}>
							<Trash2 class="h-4 w-4 text-destructive" />
						</Button>
					</div>
				</div>
			</Card.Header>
			<Card.Content>
				<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
					<div>
						<p class="text-sm text-muted-foreground">Image</p>
						<p class="font-medium">{container.image}</p>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">Status</p>
						<p class="font-medium capitalize">{container.state} - {container.status}</p>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">Container ID</p>
						<p class="font-medium font-mono text-sm">{container.id.slice(0, 12)}</p>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">Created</p>
						<p class="font-medium">{new Date(container.created * 1000).toLocaleString()}</p>
					</div>
				</div>
				{#if container.ports && container.ports.length > 0}
					<div class="mt-4">
						<p class="text-sm text-muted-foreground mb-1">Ports</p>
						<div class="flex gap-2">
							{#each container.ports as port}
								<span class="px-2 py-1 bg-muted rounded text-sm">
									{port.public_port || '?'}:{port.private_port}/{port.port_type}
								</span>
							{/each}
						</div>
					</div>
				{/if}
			</Card.Content>
		</Card.Root>

		<!-- Stats -->
		{#if stats && container.state === 'running'}
			<Card.Root>
				<Card.Header>
					<div class="flex items-center justify-between">
						<Card.Title>Resource Usage</Card.Title>
						<Button variant="ghost" size="sm" onclick={loadStats}>
							<RefreshCw class="h-4 w-4" />
						</Button>
					</div>
				</Card.Header>
				<Card.Content>
					<div class="grid gap-4 md:grid-cols-4">
						<div>
							<p class="text-sm text-muted-foreground">CPU</p>
							<p class="text-2xl font-bold">{stats.cpu_percent.toFixed(1)}%</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Memory</p>
							<p class="text-2xl font-bold">{stats.memory_percent.toFixed(1)}%</p>
							<p class="text-xs text-muted-foreground">
								{formatBytes(stats.memory_usage)} / {formatBytes(stats.memory_limit)}
							</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Network RX</p>
							<p class="text-2xl font-bold">{formatBytes(stats.network_rx)}</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Network TX</p>
							<p class="text-2xl font-bold">{formatBytes(stats.network_tx)}</p>
						</div>
					</div>
				</Card.Content>
			</Card.Root>
		{/if}

		<!-- Logs -->
		<Card.Root>
			<Card.Header>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Terminal class="h-5 w-5" />
						<Card.Title>Logs</Card.Title>
					</div>
					<Button variant="outline" size="sm" onclick={loadLogs} disabled={logsLoading}>
						<RefreshCw class="h-4 w-4 mr-1 {logsLoading ? 'animate-spin' : ''}" />
						Refresh
					</Button>
				</div>
			</Card.Header>
			<Card.Content>
				<div class="bg-black rounded-lg p-4 max-h-96 overflow-auto font-mono text-sm text-green-400">
					{#if logs.length === 0}
						<p class="text-muted-foreground">No logs available</p>
					{:else}
						{#each logs as line}
							<div class="whitespace-pre-wrap break-all">{line}</div>
						{/each}
					{/if}
				</div>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
