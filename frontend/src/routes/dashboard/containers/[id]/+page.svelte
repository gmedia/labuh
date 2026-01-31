<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { ArrowLeft } from '@lucide/svelte';
	import { ContainerController } from '$lib/features/containers/container-controller.svelte';
	import ContainerInfo from '$lib/features/containers/components/ContainerInfo.svelte';
	import ResourceUsage from '$lib/features/containers/components/ResourceUsage.svelte';
	import TerminalLogs from '$lib/features/containers/components/TerminalLogs.svelte';

	const containerId = $page.params.id || '';
	let ctrl = $state(new ContainerController(containerId));
	let autoRefresh = $state(true);
	let refreshInterval: any = null;

	onMount(async () => {
		await ctrl.init();

		refreshInterval = setInterval(() => {
			if (autoRefresh && ctrl.container?.state === 'running') {
				ctrl.loadStats();
			}
		}, 5000);
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});
</script>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" onclick={() => goto('/dashboard/containers')}>
			<ArrowLeft class="h-5 w-5" />
		</Button>
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Container Details</h2>
			<p class="text-muted-foreground">{containerId?.slice(0, 12)}</p>
		</div>
	</div>

	{#if ctrl.loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if !ctrl.container}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<h3 class="text-lg font-semibold">Container not found</h3>
				<p class="mb-4 text-sm text-muted-foreground">The container may have been removed.</p>
				<Button onclick={() => goto('/dashboard/containers')}>Back to Containers</Button>
			</Card.Content>
		</Card.Root>
	{:else}
		<ContainerInfo bind:ctrl />
		<ResourceUsage bind:ctrl />
		<TerminalLogs bind:ctrl />
	{/if}
</div>
