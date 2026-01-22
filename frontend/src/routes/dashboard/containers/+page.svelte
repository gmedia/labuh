<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Container } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Container as ContainerIcon, Play, Square, RotateCcw, Trash2, Plus, X } from '@lucide/svelte';

	let containers = $state<Container[]>([]);
	let loading = $state(true);
	let showCreateDialog = $state(false);
	let newContainer = $state({ name: '', image: '' });
	let creating = $state(false);
	let actionLoading = $state<string | null>(null);

	async function loadContainers() {
		loading = true;
		const result = await api.containers.list(true);
		if (result.data) {
			containers = result.data;
		}
		loading = false;
	}

	onMount(loadContainers);

	async function createContainer() {
		if (!newContainer.name || !newContainer.image) return;
		creating = true;
		const result = await api.containers.create({
			name: newContainer.name,
			image: newContainer.image,
		});
		if (result.data) {
			showCreateDialog = false;
			newContainer = { name: '', image: '' };
			await loadContainers();
		} else {
			alert(result.message || 'Failed to create container');
		}
		creating = false;
	}

	async function startContainer(id: string) {
		actionLoading = id;
		await api.containers.start(id);
		await loadContainers();
		actionLoading = null;
	}

	async function stopContainer(id: string) {
		actionLoading = id;
		await api.containers.stop(id);
		await loadContainers();
		actionLoading = null;
	}

	async function restartContainer(id: string) {
		actionLoading = id;
		await api.containers.restart(id);
		await loadContainers();
		actionLoading = null;
	}

	async function removeContainer(id: string) {
		if (!confirm('Are you sure you want to delete this container?')) return;
		actionLoading = id;
		await api.containers.remove(id);
		await loadContainers();
		actionLoading = null;
	}

	function getStatusColor(state: string): string {
		switch (state) {
			case 'running': return 'text-green-500';
			case 'exited': return 'text-red-500';
			case 'paused': return 'text-yellow-500';
			default: return 'text-muted-foreground';
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Containers</h2>
			<p class="text-muted-foreground">Manage your running containers</p>
		</div>
		<Button class="gap-2" onclick={() => showCreateDialog = true}>
			<Plus class="h-4 w-4" />
			Create Container
		</Button>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if containers.length === 0}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<ContainerIcon class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No containers yet</h3>
				<p class="mb-4 text-sm text-muted-foreground">
					Create a container from an image to get started
				</p>
				<Button onclick={() => showCreateDialog = true}>Create Container</Button>
			</Card.Content>
		</Card.Root>
	{:else}
		<div class="grid gap-4">
			{#each containers as container}
				<Card.Root>
					<Card.Content class="flex items-center justify-between p-4">
						<div class="flex items-center gap-4">
							<ContainerIcon class="h-8 w-8 text-muted-foreground" />
							<a href="/dashboard/containers/{container.id}" class="hover:underline">
								<p class="font-medium">{container.names[0]?.replace(/^\//, '') || container.id.slice(0, 12)}</p>
								<p class="text-sm text-muted-foreground">{container.image}</p>
								<p class="text-xs {getStatusColor(container.state)} capitalize">{container.state} - {container.status}</p>
							</a>
						</div>
						<div class="flex items-center gap-2">
							{#if container.state !== 'running'}
								<Button variant="outline" size="icon" onclick={() => startContainer(container.id)} disabled={actionLoading === container.id}>
									<Play class="h-4 w-4" />
								</Button>
							{:else}
								<Button variant="outline" size="icon" onclick={() => stopContainer(container.id)} disabled={actionLoading === container.id}>
									<Square class="h-4 w-4" />
								</Button>
							{/if}
							<Button variant="outline" size="icon" onclick={() => restartContainer(container.id)} disabled={actionLoading === container.id}>
								<RotateCcw class="h-4 w-4" />
							</Button>
							<Button variant="outline" size="icon" onclick={() => removeContainer(container.id)} disabled={actionLoading === container.id}>
								<Trash2 class="h-4 w-4 text-destructive" />
							</Button>
						</div>
					</Card.Content>
				</Card.Root>
			{/each}
		</div>
	{/if}
</div>

<!-- Create Dialog -->
{#if showCreateDialog}
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
		<Card.Root class="w-full max-w-md mx-4">
			<Card.Header>
				<div class="flex items-center justify-between">
					<Card.Title>Create Container</Card.Title>
					<Button variant="ghost" size="icon" onclick={() => showCreateDialog = false}>
						<X class="h-4 w-4" />
					</Button>
				</div>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="space-y-2">
					<Label for="name">Container Name</Label>
					<Input id="name" placeholder="my-container" bind:value={newContainer.name} />
				</div>
				<div class="space-y-2">
					<Label for="image">Image</Label>
					<Input id="image" placeholder="nginx:latest" bind:value={newContainer.image} />
				</div>
			</Card.Content>
			<Card.Footer class="flex justify-end gap-2">
				<Button variant="outline" onclick={() => showCreateDialog = false}>Cancel</Button>
				<Button onclick={createContainer} disabled={creating || !newContainer.name || !newContainer.image}>
					{creating ? 'Creating...' : 'Create'}
				</Button>
			</Card.Footer>
		</Card.Root>
	</div>
{/if}
