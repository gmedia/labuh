<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { toast } from 'svelte-sonner';
	import { api } from '$lib/api';
	import { activeTeam } from '$lib/stores';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Layers, Plus, Upload, Users } from '@lucide/svelte';
	import { StackListController } from '$lib/features/stacks/stack-list-controller.svelte';
	import StackCard from '$lib/features/stacks/components/StackCard.svelte';
	import CreateStackDialog from '$lib/features/stacks/components/CreateStackDialog.svelte';

	let ctrl = $state(new StackListController());
	let fileInput = $state<HTMLInputElement>();

	async function checkTemplate() {
		const templateId = $page.url.searchParams.get('template');
		if (templateId) {
			const result = await api.templates.get(templateId);
			if (result.data) {
				ctrl.newStack.name = result.data.id;
				ctrl.newStack.composeContent = result.data.compose_content;
				ctrl.showCreateDialog = true;

				// Clear the URL parameter without reloading
				const newUrl = new URL($page.url);
				newUrl.searchParams.delete('template');
				window.history.replaceState({}, '', newUrl);
			}
		}
	}

	onMount(async () => {
		await ctrl.init();
		await checkTemplate();
	});

	$effect(() => {
		if ($activeTeam) {
			ctrl.loadStacks();
		}
	});

	async function handleRestore(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		const reader = new FileReader();
		reader.onload = async (e) => {
			try {
				const backup = JSON.parse(e.target?.result as string);
				await ctrl.restoreBackup(backup);
			} catch (err: any) {
				toast.error('Invalid backup file format');
			} finally {
				target.value = ''; // Reset input
			}
		};
		reader.readAsText(file);
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Stacks</h2>
			<p class="text-muted-foreground">Deploy from Docker Compose files</p>
		</div>
		<div class="flex items-center gap-2">
			<input type="file" accept=".json" class="hidden" onchange={handleRestore} bind:this={fileInput} disabled={ctrl.creating || !$activeTeam?.team || $activeTeam.role === 'Viewer'} />
			<Button
				variant="outline"
				class="gap-2"
				onclick={() => fileInput?.click()}
				disabled={ctrl.creating || !$activeTeam?.team || $activeTeam.role === 'Viewer'}
			>
				<Upload class="h-4 w-4" />
				Restore Backup
			</Button>
			<Button
				class="gap-2"
				onclick={() => ctrl.showCreateDialog = true}
				disabled={!$activeTeam?.team || $activeTeam.role === 'Viewer'}
			>
				<Plus class="h-4 w-4" />
				Import Compose
			</Button>
		</div>
	</div>

	{#if !$activeTeam?.team}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<Users class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No team selected</h3>
				<p class="mb-4 text-sm text-muted-foreground">
					Please select or create a team to manage stacks.
				</p>
				<Button href="/dashboard/teams">Go to Teams</Button>
			</Card.Content>
		</Card.Root>
	{:else if ctrl.loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if ctrl.stacks.length === 0}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<Layers class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No stacks yet</h3>
				<p class="mb-4 text-sm text-muted-foreground">
					Import a docker-compose.yml to create your first stack
				</p>
				<Button onclick={() => ctrl.showCreateDialog = true}>Import Compose</Button>
			</Card.Content>
		</Card.Root>
	{:else}
		<div class="grid gap-4">
			{#each ctrl.stacks as stack}
				<StackCard {stack} {ctrl} />
			{/each}
		</div>
	{/if}
</div>

<CreateStackDialog bind:ctrl />
