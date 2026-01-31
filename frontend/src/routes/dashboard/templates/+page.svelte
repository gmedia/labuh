<script lang="ts">
	import { onMount } from 'svelte';
	import { activeTeam } from '$lib/stores';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { LayoutGrid, Search, Users, Plus } from '@lucide/svelte';
	import { TemplateController } from '$lib/features/templates/template-controller.svelte';
	import TemplateCard from '$lib/features/templates/components/TemplateCard.svelte';
	import AddTemplateDialog from '$lib/features/templates/components/AddTemplateDialog.svelte';

	let ctrl = $state(new TemplateController());

	onMount(() => {
		ctrl.init();
	});
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">App Templates</h2>
			<p class="text-muted-foreground">One-click deployment for popular applications</p>
		</div>
		<Button onclick={() => ctrl.showAddDialog = true} class="gap-2">
			<Plus class="h-4 w-4" />
			Add Template
		</Button>
	</div>

	<div class="relative">
		<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
		<input
			type="text"
			placeholder="Search templates..."
			bind:value={ctrl.searchQuery}
			class="w-full pl-9 pr-4 py-2 bg-background border rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
		/>
	</div>

	{#if ctrl.loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if ctrl.filteredTemplates.length === 0}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<LayoutGrid class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No templates found</h3>
				<p class="text-sm text-muted-foreground">
					{ctrl.searchQuery ? 'Try a different search term' : 'Templates are not available at the moment'}
				</p>
			</Card.Content>
		</Card.Root>
	{:else}
		<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
			{#each ctrl.filteredTemplates as template}
				<TemplateCard {template} />
			{/each}
		</div>
	{/if}

	{#if !$activeTeam?.team}
		<div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-900/50 p-4 rounded-lg flex items-start gap-3 mt-6">
			<Users class="h-5 w-5 text-yellow-600 dark:text-yellow-500 mt-0.5" />
			<div>
				<p class="text-sm font-medium text-yellow-800 dark:text-yellow-400">Team Required</p>
				<p class="text-sm text-yellow-700 dark:text-yellow-500/80">
					You need to select a team before you can deploy templates.
					<a href="/dashboard/teams" class="font-bold underline">Go to Teams</a>
				</p>
			</div>
		</div>
	{/if}
</div>

<AddTemplateDialog bind:ctrl />
