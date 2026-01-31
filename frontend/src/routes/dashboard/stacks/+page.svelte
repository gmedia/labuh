<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { toast } from 'svelte-sonner';
	import { api, type Stack } from '$lib/api';
	import { activeTeam, auth } from '$lib/stores';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Layers, Play, Square, Trash2, Plus, X, FileCode, Users, Upload, GitBranch, Globe } from '@lucide/svelte';

	let stacks = $state<Stack[]>([]);
	let loading = $state(true);
	let showCreateDialog = $state(false);
	let newStack = $state({ name: '', composeContent: '' });
	let creating = $state(false);
	let actionLoading = $state<string | null>(null);
	let fileInput = $state<HTMLInputElement>();

	let importMode = $state<'manual' | 'git'>('manual');
	let gitStack = $state({
		url: '',
		branch: 'main',
		composePath: 'docker-compose.yml'
	});

	async function loadStacks() {
		if (!$activeTeam?.team) {
			stacks = [];
			loading = false;
			return;
		}
		loading = true;
		const result = await api.stacks.list($activeTeam.team.id);
		if (result.data) {
			stacks = result.data;
		}
		loading = false;
	}

	async function checkTemplate() {
		const templateId = $page.url.searchParams.get('template');
		if (templateId) {
			const result = await api.templates.get(templateId);
			if (result.data) {
				newStack.name = result.data.id;
				newStack.composeContent = result.data.compose_content;
				showCreateDialog = true;

				// Clear the URL parameter without reloading
				const newUrl = new URL($page.url);
				newUrl.searchParams.delete('template');
				window.history.replaceState({}, '', newUrl);
			}
		}
	}

	onMount(async () => {
		await loadStacks();
		await checkTemplate();
	});

	$effect(() => {
		if ($activeTeam) {
			loadStacks();
		}
	});

	async function createStack() {
		if (!newStack.name || !$activeTeam?.team) return;
		creating = true;

		let result;
		if (importMode === 'manual') {
			if (!newStack.composeContent) {
				creating = false;
				return;
			}
			result = await api.stacks.create({
				name: newStack.name,
				team_id: $activeTeam.team.id,
				compose_content: newStack.composeContent,
			});
		} else {
			if (!gitStack.url) {
				creating = false;
				return;
			}
			result = await api.stacks.createFromGit({
				name: newStack.name,
				team_id: $activeTeam.team.id,
				git_url: gitStack.url,
				git_branch: gitStack.branch,
				compose_path: gitStack.composePath,
			});
		}

		if (result.data) {
			toast.success('Stack created successfully');
			showCreateDialog = false;
			newStack = { name: '', composeContent: '' };
			gitStack = { url: '', branch: 'main', composePath: 'docker-compose.yml' };
			await loadStacks();
		} else {
			toast.error(result.message || result.error || 'Failed to create stack');
		}
		creating = false;
	}

	async function startStack(id: string) {
		actionLoading = id;
		const result = await api.stacks.start(id);
		if (!result.error) toast.success('Stack started');
		else toast.error(result.message || result.error);
		await loadStacks();
		actionLoading = null;
	}

	async function stopStack(id: string) {
		actionLoading = id;
		const result = await api.stacks.stop(id);
		if (!result.error) toast.success('Stack stopped');
		else toast.error(result.message || result.error);
		await loadStacks();
		actionLoading = null;
	}

	async function removeStack(id: string) {
		if (!confirm('Are you sure you want to delete this stack and all its containers?')) return;
		actionLoading = id;
		const result = await api.stacks.remove(id);
		if (!result.error) toast.success('Stack removed');
		else toast.error(result.message || result.error);
		await loadStacks();
		actionLoading = null;
	}

	async function handleRestore(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file || !$activeTeam?.team) return;

		const reader = new FileReader();
		reader.onload = async (e) => {
			try {
				const backup = JSON.parse(e.target?.result as string);
				creating = true;
				const result = await api.stacks.restore($activeTeam.team!.id, backup);
				if (result.data) {
					toast.success('Stack restored successfully');
					await loadStacks();
				} else {
					toast.error(result.message || result.error || 'Failed to restore stack');
				}
			} catch (err: any) {
				toast.error('Invalid backup file format');
			} finally {
				creating = false;
				target.value = ''; // Reset input
			}
		};
		reader.readAsText(file);
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
		<div class="flex items-center gap-2">
			<input type="file" accept=".json" class="hidden" onchange={handleRestore} bind:this={fileInput} disabled={creating || !$activeTeam?.team || $activeTeam.role === 'Viewer'} />
			<Button
				variant="outline"
				class="gap-2"
				onclick={() => fileInput?.click()}
				disabled={creating || !$activeTeam?.team || $activeTeam.role === 'Viewer'}
			>
				<Upload class="h-4 w-4" />
				Restore Backup
			</Button>
			<Button
				class="gap-2"
				onclick={() => showCreateDialog = true}
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
	{:else if loading}
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
							{#if $activeTeam.role !== 'Viewer'}
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
							{:else}
								<span class="text-xs text-muted-foreground italic px-2">Read Only</span>
							{/if}
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
				<div class="space-y-4">
					<div class="flex p-1 bg-muted rounded-lg">
						<button
							class="flex-1 flex items-center justify-center gap-2 py-1.5 text-sm font-medium rounded-md transition-all {importMode === 'manual' ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
							onclick={() => importMode = 'manual'}
						>
							<FileCode class="h-4 w-4" />
							Manual
						</button>
						<button
							class="flex-1 flex items-center justify-center gap-2 py-1.5 text-sm font-medium rounded-md transition-all {importMode === 'git' ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
							onclick={() => importMode = 'git'}
						>
							<GitBranch class="h-4 w-4" />
							Git
						</button>
					</div>

					{#if importMode === 'manual'}
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
								rows={10}
								class="font-mono text-sm"
							/>
						</div>
					{:else}
						<div class="space-y-4">
							<div class="space-y-2">
								<Label for="gitUrl">Repository URL</Label>
								<Input id="gitUrl" placeholder="https://github.com/user/repo" bind:value={gitStack.url} />
							</div>
							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-2">
									<Label for="gitBranch">Branch</Label>
									<Input id="gitBranch" placeholder="main" bind:value={gitStack.branch} />
								</div>
								<div class="space-y-2">
									<Label for="composePath">Compose Path</Label>
									<Input id="composePath" placeholder="docker-compose.yml" bind:value={gitStack.composePath} />
								</div>
							</div>
						</div>
					{/if}
				</div>
				<div class="text-sm text-muted-foreground">
					<p><strong>Note:</strong> Only images are supported. Build contexts are not available.</p>
				</div>
			</Card.Content>
			<Card.Footer class="flex justify-end gap-2">
				<Button variant="outline" onclick={() => showCreateDialog = false}>Cancel</Button>
				<Button
					onclick={createStack}
					disabled={creating || !newStack.name || (importMode === 'manual' ? !newStack.composeContent : !gitStack.url)}
				>
					{creating ? 'Creating...' : 'Create Stack'}
				</Button>
			</Card.Footer>
		</Card.Root>
	</div>
{/if}
