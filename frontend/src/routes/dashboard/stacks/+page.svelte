<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { api, type Stack } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Layers, Play, Square, Trash2, Plus, X, FileCode } from '@lucide/svelte';

	let stacks = $state<Stack[]>([]);
	let loading = $state(true);
	let showCreateDialog = $state(false);
	let newStack = $state({ name: '', composeContent: '' });
	let creating = $state(false);
	let actionLoading = $state<string | null>(null);

	async function loadStacks() {
		loading = true;
		const result = await api.stacks.list();
		if (result.data) {
			stacks = result.data;
		}
		loading = false;
	}

	onMount(loadStacks);

	async function createStack() {
		if (!newStack.name || !newStack.composeContent) return;
		creating = true;
		const result = await api.stacks.create({
			name: newStack.name,
			compose_content: newStack.composeContent,
		});
		if (result.data) {
			toast.success('Stack created successfully');
			showCreateDialog = false;
			newStack = { name: '', composeContent: '' };
			await loadStacks();
		} else {
			toast.error(result.error || 'Failed to create stack');
		}
		creating = false;
	}

	async function startStack(id: string) {
		actionLoading = id;
		const result = await api.stacks.start(id);
		if (!result.error) toast.success('Stack started');
		else toast.error(result.error);
		await loadStacks();
		actionLoading = null;
	}

	async function stopStack(id: string) {
		actionLoading = id;
		const result = await api.stacks.stop(id);
		if (!result.error) toast.success('Stack stopped');
		else toast.error(result.error);
		await loadStacks();
		actionLoading = null;
	}

	async function removeStack(id: string) {
		if (!confirm('Are you sure you want to delete this stack and all its containers?')) return;
		actionLoading = id;
		const result = await api.stacks.remove(id);
		if (!result.error) toast.success('Stack removed');
		else toast.error(result.error);
		await loadStacks();
		actionLoading = null;
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'running': return 'text-green-500';
			case 'stopped': return 'text-red-500';
			case 'creating': return 'text-yellow-500';
			default: return 'text-muted-foreground';
		}
	}

	const sampleCompose = `version: '3.8'
services:
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
  redis:
    image: redis:alpine`;
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Stacks</h2>
			<p class="text-muted-foreground">Deploy from Docker Compose files</p>
		</div>
		<Button class="gap-2" onclick={() => showCreateDialog = true}>
			<Plus class="h-4 w-4" />
			Import Compose
		</Button>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if stacks.length === 0}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<Layers class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No stacks yet</h3>
				<p class="mb-4 text-sm text-muted-foreground">
					Import a docker-compose.yml to create your first stack
				</p>
				<Button onclick={() => showCreateDialog = true}>Import Compose</Button>
			</Card.Content>
		</Card.Root>
	{:else}
		<div class="grid gap-4">
			{#each stacks as stack}
				<Card.Root>
					<Card.Content class="flex items-center justify-between p-4">
						<div class="flex items-center gap-4">
							<Layers class="h-8 w-8 text-muted-foreground" />
							<a href="/dashboard/stacks/{stack.id}" class="hover:underline">
								<p class="font-medium">{stack.name}</p>
								<p class="text-xs {getStatusColor(stack.status)} capitalize">{stack.status}</p>
								<p class="text-xs text-muted-foreground">
									Created {new Date(stack.created_at).toLocaleDateString()}
								</p>
							</a>
						</div>
						<div class="flex items-center gap-2">
							{#if stack.status !== 'running'}
								<Button variant="outline" size="icon" onclick={() => startStack(stack.id)} disabled={actionLoading === stack.id}>
									<Play class="h-4 w-4" />
								</Button>
							{:else}
								<Button variant="outline" size="icon" onclick={() => stopStack(stack.id)} disabled={actionLoading === stack.id}>
									<Square class="h-4 w-4" />
								</Button>
							{/if}
							<Button variant="outline" size="icon" onclick={() => removeStack(stack.id)} disabled={actionLoading === stack.id}>
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
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto py-8">
		<Card.Root class="w-full max-w-2xl mx-4">
			<Card.Header>
				<div class="flex items-center justify-between">
					<Card.Title class="flex items-center gap-2">
						<FileCode class="h-5 w-5" />
						Import Docker Compose
					</Card.Title>
					<Button variant="ghost" size="icon" onclick={() => showCreateDialog = false}>
						<X class="h-4 w-4" />
					</Button>
				</div>
				<Card.Description>
					Paste your docker-compose.yml content to create a new stack.
				</Card.Description>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="space-y-2">
					<Label for="stackName">Stack Name</Label>
					<Input id="stackName" placeholder="my-stack" bind:value={newStack.name} />
				</div>
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label for="compose">docker-compose.yml</Label>
						<Button
							variant="ghost"
							size="sm"
							onclick={() => newStack.composeContent = sampleCompose}
						>
							Load Example
						</Button>
					</div>
					<Textarea
						id="compose"
						placeholder={sampleCompose}
						bind:value={newStack.composeContent}
						rows={12}
						class="font-mono text-sm"
					/>
				</div>
				<div class="text-sm text-muted-foreground">
					<p><strong>Note:</strong> Only images are supported. Build contexts are not available.</p>
				</div>
			</Card.Content>
			<Card.Footer class="flex justify-end gap-2">
				<Button variant="outline" onclick={() => showCreateDialog = false}>Cancel</Button>
				<Button
					onclick={createStack}
					disabled={creating || !newStack.name || !newStack.composeContent}
				>
					{creating ? 'Creating...' : 'Create Stack'}
				</Button>
			</Card.Footer>
		</Card.Root>
	</div>
{/if}
