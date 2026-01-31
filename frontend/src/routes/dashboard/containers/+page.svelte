<script lang="ts">
	import { onMount } from 'svelte';
	import { activeTeam } from '$lib/stores';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Container as ContainerIcon, Plus, Users } from '@lucide/svelte';
	import { ContainerListController } from '$lib/features/containers/container-list-controller.svelte';
	import ContainerCard from '$lib/features/containers/components/ContainerCard.svelte';
	import CreateContainerDialog from '$lib/features/containers/components/CreateContainerDialog.svelte';

	let ctrl = $state(new ContainerListController());

	onMount(() => {
		ctrl.init();
	});

	$effect(() => {
		if ($activeTeam) {
			ctrl.loadContainers();
		}
	});

	const isViewer = $derived($activeTeam?.role === 'Viewer');
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Containers</h2>
			<p class="text-muted-foreground">Manage your running containers</p>
		</div>
		<Button
			class="gap-2"
			onclick={() => ctrl.showCreateDialog = true}
			disabled={!$activeTeam?.team || isViewer}
		>
			<Plus class="h-4 w-4" />
			Create Container
		</Button>
	</div>

	{#if !$activeTeam?.team}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<Users class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No team selected</h3>
				<p class="mb-4 text-sm text-muted-foreground">
					Please select or create a team to see containers.
				</p>
				<Button href="/dashboard/teams">Go to Teams</Button>
			</Card.Content>
		</Card.Root>
	{:else if ctrl.loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if ctrl.containers.length === 0}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<ContainerIcon class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No containers yet</h3>
				<p class="mb-4 text-sm text-muted-foreground">
					Create a container from an image to get started
				</p>
				<Button onclick={() => ctrl.showCreateDialog = true}>Create Container</Button>
			</Card.Content>
		</Card.Root>
	{:else}
		<div class="grid gap-4">
			{#each ctrl.containers as container}
				<ContainerCard {container} {ctrl} />
			{/each}
		</div>
	{/if}
</div>

<CreateContainerDialog bind:ctrl />
