<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Container } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Terminal, Search, Download, RefreshCw } from '@lucide/svelte';

	let containers = $state<Container[]>([]);
	let selectedContainer = $state<string | null>(null);
	let logs = $state<string[]>([]);
	let loading = $state(true);
	let logsLoading = $state(false);
	let searchQuery = $state('');

	async function loadContainers() {
		const result = await api.containers.list(true);
		if (result.data) {
			containers = result.data;
			if (containers.length > 0 && !selectedContainer) {
				selectedContainer = containers[0].id;
				await loadLogs();
			}
		}
		loading = false;
	}

	async function loadLogs() {
		if (!selectedContainer) return;
		logsLoading = true;
		const result = await api.containers.logs(selectedContainer, 500);
		if (result.data) {
			logs = result.data;
		}
		logsLoading = false;
	}

	onMount(loadContainers);

	async function selectContainer(id: string) {
		selectedContainer = id;
		await loadLogs();
	}

	function downloadLogs() {
		const content = logs.join('\n');
		const blob = new Blob([content], { type: 'text/plain' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `logs-${selectedContainer?.slice(0, 12)}-${Date.now()}.txt`;
		a.click();
		URL.revokeObjectURL(url);
	}

	let filteredLogs = $derived(
		searchQuery
			? logs.filter(line => line.toLowerCase().includes(searchQuery.toLowerCase()))
			: logs
	);

	function getContainerName(c: Container): string {
		return c.names[0]?.replace(/^\//, '') || c.id.slice(0, 12);
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Logs</h2>
			<p class="text-muted-foreground">View and search container logs</p>
		</div>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if containers.length === 0}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<Terminal class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No containers available</h3>
				<p class="text-sm text-muted-foreground">Create some containers first to view their logs.</p>
			</Card.Content>
		</Card.Root>
	{:else}
		<div class="grid gap-4 lg:grid-cols-4">
			<!-- Container Selector -->
			<Card.Root class="lg:col-span-1">
				<Card.Header class="pb-2">
					<Card.Title class="text-sm">Containers</Card.Title>
				</Card.Header>
				<Card.Content class="p-0">
					<div class="space-y-1 p-2">
						{#each containers as container}
							<button
								class="w-full text-left px-3 py-2 rounded-lg transition-colors {selectedContainer === container.id ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
								onclick={() => selectContainer(container.id)}
							>
								<p class="font-medium text-sm truncate">{getContainerName(container)}</p>
								<p class="text-xs opacity-75 capitalize">{container.state}</p>
							</button>
						{/each}
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Logs Viewer -->
			<Card.Root class="lg:col-span-3">
				<Card.Header>
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-2">
							<Terminal class="h-5 w-5" />
							<Card.Title>
								{containers.find(c => c.id === selectedContainer)?.names[0]?.replace(/^\//, '') || 'Logs'}
							</Card.Title>
						</div>
						<div class="flex items-center gap-2">
							<Button variant="outline" size="sm" onclick={loadLogs} disabled={logsLoading}>
								<RefreshCw class="h-4 w-4 {logsLoading ? 'animate-spin' : ''}" />
							</Button>
							<Button variant="outline" size="sm" onclick={downloadLogs} disabled={logs.length === 0}>
								<Download class="h-4 w-4" />
							</Button>
						</div>
					</div>
					<div class="relative">
						<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
						<Input
							placeholder="Search logs..."
							class="pl-9"
							bind:value={searchQuery}
						/>
					</div>
				</Card.Header>
				<Card.Content>
					<div class="bg-black rounded-lg p-4 max-h-[500px] overflow-auto font-mono text-sm text-green-400">
						{#if filteredLogs.length === 0}
							<p class="text-muted-foreground">
								{searchQuery ? 'No matching logs found' : 'No logs available'}
							</p>
						{:else}
							{#each filteredLogs as line, i}
								<div class="whitespace-pre-wrap break-all hover:bg-white/5 px-1">
									<span class="text-muted-foreground mr-2 select-none">{i + 1}</span>
									{line}
								</div>
							{/each}
						{/if}
					</div>
					<p class="mt-2 text-xs text-muted-foreground">
						Showing {filteredLogs.length} of {logs.length} lines
					</p>
				</Card.Content>
			</Card.Root>
		</div>
	{/if}
</div>
