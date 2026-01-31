<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { StackController } from '$lib/features/stacks/stack-controller.svelte';

	// Components
	import StackHeader from '$lib/features/stacks/components/StackHeader.svelte';
	import ContainerList from '$lib/features/stacks/components/ContainerList.svelte';
	import EnvironmentEditor from '$lib/features/stacks/components/EnvironmentEditor.svelte';
	import ResourceMonitoring from '$lib/features/stacks/components/ResourceMonitoring.svelte';
	import ResourceLimits from '$lib/features/stacks/components/ResourceLimits.svelte';
	import StackAutomation from '$lib/features/stacks/components/StackAutomation.svelte';
	import StackDomains from '$lib/features/stacks/components/StackDomains.svelte';
	import StackWebhooks from '$lib/features/stacks/components/StackWebhooks.svelte';
	import DeploymentHistory from '$lib/features/stacks/components/DeploymentHistory.svelte';
	import ComposeEditor from '$lib/features/stacks/components/ComposeEditor.svelte';
	import LogViewer from '$lib/features/stacks/components/LogViewer.svelte';
	import StackHealth from '$lib/features/stacks/components/StackHealth.svelte';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { goto } from '$app/navigation';

	const stackId = $page.params.id || '';
	let ctrl = $state(new StackController(stackId));

	onMount(() => {
		ctrl.init();

		const interval = setInterval(() => {
			if (!ctrl.loading) {
				ctrl.loadContainers();
				ctrl.loadHealth();
				ctrl.loadMetrics();
			}
		}, 30000);

		return () => clearInterval(interval);
	});
</script>

<div class="space-y-6">
	{#if ctrl.loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if !ctrl.stack}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<h3 class="text-lg font-semibold">Stack not found</h3>
				<p class="mb-4 text-sm text-muted-foreground">The stack may have been removed.</p>
				<Button onclick={() => goto('/dashboard/stacks')}>Back to Stacks</Button>
			</Card.Content>
		</Card.Root>
	{:else}
		<StackHeader bind:ctrl />

		<div class="grid gap-6 md:grid-cols-3">
			<!-- Main Content -->
			<div class="md:col-span-2 space-y-6">
				<ComposeEditor bind:ctrl />
				<ContainerList bind:ctrl />
				<ResourceMonitoring bind:ctrl />
				<ResourceLimits bind:ctrl />
				<LogViewer bind:ctrl />
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<StackHealth bind:ctrl />
				<StackAutomation bind:ctrl />
				<EnvironmentEditor bind:ctrl />
				<StackWebhooks bind:ctrl />
				<StackDomains bind:ctrl />
				<DeploymentHistory bind:ctrl />
			</div>
		</div>
	{/if}
</div>
