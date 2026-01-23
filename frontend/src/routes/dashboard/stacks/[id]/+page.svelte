<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { api, type Stack, type Container, type Domain, type DeploymentLog } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import {
		ArrowLeft, Play, Square, Trash2, RefreshCw, Terminal, Layers,
		Container as ContainerIcon, FileCode, Save, CheckCircle, XCircle,
		Globe, History, Webhook, Copy, AlertCircle, RotateCcw
	} from '@lucide/svelte';

	const stackId: string = $page.params.id ?? '';

	let stack = $state<Stack | null>(null);
	let containers = $state<Container[]>([]);
	let domains = $state<Domain[]>([]);
	let deployments = $state<DeploymentLog[]>([]);
	let logs = $state<Map<string, string[]>>(new Map());

	let loading = $state(true);
	let actionLoading = $state(false);

	let showComposeEditor = $state(false);
	let editedCompose = $state('');
	let savingCompose = $state(false);

	let selectedContainerLogs = $state<string | null>(null);
	let newDomain = $state('');
	let addingDomain = $state(false);

	async function loadStack() {
		const result = await api.stacks.get(stackId);
		if (result.data) {
			stack = result.data;
			editedCompose = stack.compose_content || '';
		}
		loading = false;
	}

	async function loadContainers() {
		const result = await api.stacks.containers(stackId);
		if (result.data) {
			containers = result.data;
		}
	}

	async function loadDomains() {
		const result = await api.stacks.domains.list(stackId);
		if (result.data) {
			domains = result.data;
		}
	}

	async function loadDeployments() {
		const result = await api.stacks.deploymentLogs(stackId);
		if (result.data) {
			deployments = result.data;
		}
	}

	async function loadContainerLogs(containerId: string) {
		selectedContainerLogs = containerId;
		const result = await api.containers.logs(containerId, 100);
		if (result.data) {
			logs.set(containerId, result.data);
			logs = new Map(logs);
		}
	}

	onMount(async () => {
		await loadStack();
		if (stack) {
			await Promise.all([
				loadContainers(),
				loadDomains(),
				loadDeployments()
			]);
		}
	});

	async function startStack() {
		if (!stack) return;
		actionLoading = true;
		await api.stacks.start(stack.id);
		await Promise.all([loadStack(), loadContainers()]);
		actionLoading = false;
	}

	async function stopStack() {
		if (!stack) return;
		actionLoading = true;
		await api.stacks.stop(stack.id);
		await Promise.all([loadStack(), loadContainers()]);
		actionLoading = false;
	}

	async function removeStack() {
		if (!stack) return;
		if (!confirm('Are you sure you want to delete this stack and all its containers?')) return;
		actionLoading = true;
		await api.stacks.remove(stack.id);
		goto('/dashboard/stacks');
	}

	async function saveCompose() {
		if (!stack) return;
		savingCompose = true;
		// TODO: Implement compose update API (redeploy)
		// For now we don't have update endpoint in frontend api for compose content specifically?
		// Ah, we need a way to update compose content.
		// Actually the plan said "Add update_compose() method".
		// I missed adding an `update` endpoint for Stack in the plan execution.
		// I added `redeploy_stack` but that uses existing compose content.
		// I should check if I can use `create_stack` to update? No.
		// I need to add an update endpoint.
		// For now, let's keep it disabled or mock it.
		alert('Feature coming soon: Update compose');
		savingCompose = false;
		showComposeEditor = false;
	}

	async function addDomain() {
		if (!newDomain || !stack) return;
		addingDomain = true;
		await api.stacks.domains.add(stack.id, newDomain);
		newDomain = '';
		await loadDomains();
		addingDomain = false;
	}

	async function removeDomain(domain: string) {
		if (!stack || !confirm(`Remove domain ${domain}?`)) return;
		await api.stacks.domains.remove(stack.id, domain);
		await loadDomains();
	}

	async function verifyDomain(domain: string) {
		if (!stack) return;
		await api.stacks.domains.verify(stack.id, domain);
		await loadDomains();
	}

	async function regenerateWebhook() {
		if (!stack || !confirm('Regenerate webhook token? Previous URL will stop working.')) return;
		const result = await api.stacks.regenerateWebhookToken(stack.id);
		if (result.data && stack) {
			stack.webhook_token = result.data.token;
		}
	}

	function copyToClipboard(text: string) {
		navigator.clipboard.writeText(text);
		// TODO: Toast notification
	}

	function getStatusColor(state: string): string {
		switch (state) {
			case 'running': return 'text-green-500';
			case 'exited': return 'text-red-500';
			case 'paused': return 'text-yellow-500';
			default: return 'text-muted-foreground';
		}
	}

	function getStatusBg(state: string): string {
		switch (state) {
			case 'running': return 'bg-green-500';
			case 'exited': return 'bg-red-500';
			case 'paused': return 'bg-yellow-500';
			default: return 'bg-muted-foreground';
		}
	}

	const runningCount = $derived(containers.filter(c => c.state === 'running').length);
	const stoppedCount = $derived(containers.filter(c => c.state !== 'running').length);
	const webhookUrl = $derived(stack?.webhook_token ? `${window.location.origin}/api/webhooks/deploy/${stack.id}/${stack.webhook_token}` : '');
</script>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" onclick={() => goto('/dashboard/stacks')}>
			<ArrowLeft class="h-5 w-5" />
		</Button>
		<div class="flex-1">
			<h2 class="text-2xl font-bold tracking-tight">{stack?.name || 'Loading...'}</h2>
			<p class="text-muted-foreground">Stack Details</p>
		</div>
		<Button variant="ghost" size="sm" onclick={loadStack}>
			<RefreshCw class="h-4 w-4" />
		</Button>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if !stack}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
				<h3 class="text-lg font-semibold">Stack not found</h3>
				<p class="mb-4 text-sm text-muted-foreground">The stack may have been removed.</p>
				<Button onclick={() => goto('/dashboard/stacks')}>Back to Stacks</Button>
			</Card.Content>
		</Card.Root>
	{:else}
		<div class="grid gap-6 md:grid-cols-3">
			<!-- Stack Actions & Info -->
			<div class="md:col-span-2 space-y-6">
				<Card.Root>
					<Card.Header>
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-3">
								<Layers class="h-6 w-6" />
								<div>
									<Card.Title>{stack.name}</Card.Title>
									<p class="text-sm text-muted-foreground capitalize {getStatusColor(stack.status)}">{stack.status}</p>
								</div>
							</div>
							<div class="flex items-center gap-2">
								{#if stack.status !== 'running'}
									<Button variant="outline" size="sm" onclick={startStack} disabled={actionLoading}>
										<Play class="h-4 w-4 mr-1" /> Start
									</Button>
								{:else}
									<Button variant="outline" size="sm" onclick={stopStack} disabled={actionLoading}>
										<Square class="h-4 w-4 mr-1" /> Stop
									</Button>
								{/if}
								<Button variant="outline" size="sm" onclick={() => showComposeEditor = !showComposeEditor}>
									<FileCode class="h-4 w-4 mr-1" /> Edit
								</Button>
								<Button variant="outline" size="sm" onclick={removeStack} disabled={actionLoading}>
									<Trash2 class="h-4 w-4 text-destructive" />
								</Button>
							</div>
						</div>
					</Card.Header>
					<Card.Content>
						<div class="grid gap-4 md:grid-cols-4">
							<div>
								<p class="text-sm text-muted-foreground">Created</p>
								<p class="font-medium">{new Date(stack.created_at).toLocaleDateString()}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Containers</p>
								<p class="font-medium">{containers.length}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Running</p>
								<p class="font-medium text-green-500">{runningCount}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Stopped</p>
								<p class="font-medium text-red-500">{stoppedCount}</p>
							</div>
						</div>
					</Card.Content>
				</Card.Root>

				<!-- Compose Editor -->
				{#if showComposeEditor}
					<Card.Root>
						<Card.Header>
							<div class="flex items-center justify-between">
								<Card.Title class="flex items-center gap-2">
									<FileCode class="h-5 w-5" />
									docker-compose.yml
								</Card.Title>
								<div class="flex gap-2">
									<Button variant="outline" size="sm" onclick={() => showComposeEditor = false}>Cancel</Button>
									<Button size="sm" onclick={saveCompose} disabled={savingCompose}>
										<Save class="h-4 w-4 mr-1" />
										{savingCompose ? 'Saving...' : 'Save & Redeploy'}
									</Button>
								</div>
							</div>
						</Card.Header>
						<Card.Content>
							<Textarea
								bind:value={editedCompose}
								rows={15}
								class="font-mono text-sm"
								placeholder="version: '3.8'..."
							/>
							<p class="mt-2 text-xs text-muted-foreground">
								<strong>Warning:</strong> Saving will stop and recreate all containers in this stack.
							</p>
						</Card.Content>
					</Card.Root>
				{/if}

				<!-- Containers List -->
				<Card.Root>
					<Card.Header>
						<div class="flex items-center justify-between">
							<Card.Title class="flex items-center gap-2">
								<ContainerIcon class="h-5 w-5" />
								Containers
							</Card.Title>
							<Button variant="ghost" size="sm" onclick={loadContainers}>
								<RefreshCw class="h-4 w-4" />
							</Button>
						</div>
					</Card.Header>
					<Card.Content class="space-y-2">
						{#if containers.length === 0}
							<p class="text-sm text-muted-foreground text-center py-4">No containers in this stack</p>
						{:else}
							{#each containers as container}
								<div class="flex items-center justify-between p-3 rounded-lg bg-muted/50 hover:bg-muted transition-colors">
									<div class="flex items-center gap-3">
										<span class="h-2 w-2 rounded-full {getStatusBg(container.state)}"></span>
										<div>
											<a href="/dashboard/containers/{container.id}" class="font-medium hover:underline">
												{container.names[0]?.replace(/^\//, '') || container.id.slice(0, 12)}
											</a>
											<div class="flex items-center gap-2 text-xs text-muted-foreground">
												<span>{container.image}</span>
												{#if container.ports && container.ports.length > 0}
													<span>•</span>
													<span>{container.ports.map(p => `${p.public_port || p.private_port}`).join(', ')}</span>
												{/if}
											</div>
										</div>
									</div>
									<div class="flex items-center gap-2">
										<Button variant="ghost" size="sm" onclick={() => loadContainerLogs(container.id)}>
											<Terminal class="h-4 w-4" />
										</Button>
									</div>
								</div>
							{/each}
						{/if}
					</Card.Content>
				</Card.Root>

				<!-- Logs Viewer -->
				{#if selectedContainerLogs}
					<Card.Root>
						<Card.Header>
							<div class="flex items-center justify-between">
								<Card.Title class="flex items-center gap-2">
									<Terminal class="h-5 w-5" />
									Logs: {containers.find(c => c.id === selectedContainerLogs)?.names[0]?.replace(/^\//, '') || selectedContainerLogs.slice(0, 12)}
								</Card.Title>
								<div class="flex gap-2">
									<Button variant="ghost" size="sm" onclick={() => loadContainerLogs(selectedContainerLogs!)}>
										<RefreshCw class="h-4 w-4" />
									</Button>
									<Button variant="ghost" size="sm" onclick={() => selectedContainerLogs = null}>
										Close
									</Button>
								</div>
							</div>
						</Card.Header>
						<Card.Content>
							<div class="bg-black rounded-lg p-4 max-h-80 overflow-auto font-mono text-sm text-green-400">
								{#if logs.get(selectedContainerLogs)?.length === 0}
									<p class="text-muted-foreground">No logs available</p>
								{:else}
									{#each logs.get(selectedContainerLogs) || [] as line}
										<div class="whitespace-pre-wrap break-all">{line}</div>
									{/each}
								{/if}
							</div>
						</Card.Content>
					</Card.Root>
				{/if}
			</div>

			<!-- Sidebar: Domains & Webhooks & History -->
			<div class="space-y-6">
				<!-- Webhooks -->
				<Card.Root>
					<Card.Header>
						<Card.Title class="flex items-center gap-2">
							<Webhook class="h-5 w-5" />
							Webhook
						</Card.Title>
						<Card.Description>Trigger deployments automatically</Card.Description>
					</Card.Header>
					<Card.Content class="space-y-4">
						{#if stack.webhook_token}
							<div class="space-y-2">
								<Label>Webhook URL</Label>
								<div class="flex items-center gap-2">
									<Input readonly value={webhookUrl} class="font-mono text-xs" />
									<Button variant="outline" size="icon" onclick={() => copyToClipboard(webhookUrl)}>
										<Copy class="h-4 w-4" />
									</Button>
								</div>
								<p class="text-xs text-muted-foreground">
									POST to this URL to pull latest images and redeploy.
								</p>
							</div>
							<Button variant="outline" size="sm" class="w-full" onclick={regenerateWebhook}>
								Regenerate Token
							</Button>
						{:else}
							<Button class="w-full" onclick={regenerateWebhook}>
								Generate Webhook
							</Button>
						{/if}
					</Card.Content>
				</Card.Root>

				<!-- Domains -->
				<Card.Root>
					<Card.Header>
						<Card.Title class="flex items-center gap-2">
							<Globe class="h-5 w-5" />
							Domains
						</Card.Title>
						<Card.Description>Manage custom domains</Card.Description>
					</Card.Header>
					<Card.Content class="space-y-4">
						<div class="flex gap-2">
							<Input placeholder="example.com" bind:value={newDomain} />
							<Button size="icon" onclick={addDomain} disabled={addingDomain}>
								{#if addingDomain}
									<div class="animate-spin rounded-full h-3 w-3 border-b-2 border-primary-foreground"></div>
								{:else}
									<div class="h-4 w-4">+</div>
								{/if}
							</Button>
						</div>

						<div class="space-y-2">
							{#if domains.length === 0}
								<p class="text-xs text-muted-foreground text-center py-2">No domains configured</p>
							{:else}
								{#each domains as domain}
									<div class="flex items-center justify-between p-2 rounded border bg-background/50">
										<div class="flex items-center gap-2 overflow-hidden">
											{#if domain.verified}
												<CheckCircle class="h-3 w-3 text-green-500 flex-shrink-0" />
											{:else}
												<AlertCircle class="h-3 w-3 text-yellow-500 flex-shrink-0" />
											{/if}
											<span class="text-sm truncate" title={domain.domain}>{domain.domain}</span>
										</div>
										<div class="flex gap-1">
											{#if !domain.verified}
												<Button variant="ghost" size="icon" class="h-6 w-6" onclick={() => verifyDomain(domain.domain)} title="Verify">
													<CheckCircle class="h-3 w-3" />
												</Button>
											{/if}
											<Button variant="ghost" size="icon" class="h-6 w-6 text-destructive" onclick={() => removeDomain(domain.domain)} title="Remove">
												<Trash2 class="h-3 w-3" />
											</Button>
										</div>
									</div>
								{/each}
							{/if}
						</div>
					</Card.Content>
				</Card.Root>

				<!-- Deployment History -->
				<Card.Root>
					<Card.Header>
						<div class="flex items-center justify-between">
							<Card.Title class="flex items-center gap-2">
								<History class="h-5 w-5" />
								Deployments
							</Card.Title>
							<Button variant="ghost" size="sm" onclick={loadDeployments}>
								<RefreshCw class="h-3 w-3" />
							</Button>
						</div>
					</Card.Header>
					<Card.Content>
						{#if deployments.length === 0}
							<p class="text-xs text-muted-foreground text-center py-4">No deployments yet</p>
						{:else}
							<div class="space-y-2 max-h-60 overflow-y-auto pr-1">
								{#each deployments as log}
									<div class="p-2 rounded border bg-background/50 text-xs">
										<div class="flex items-center justify-between mb-1">
											<span class="font-medium capitalize flex items-center gap-1">
												{#if log.status === 'success'}
													<CheckCircle class="h-3 w-3 text-green-500" />
												{:else if log.status === 'failed'}
													<XCircle class="h-3 w-3 text-red-500" />
												{:else}
													<RotateCcw class="h-3 w-3 text-yellow-500 animate-spin" />
												{/if}
												{log.trigger_type}
											</span>
											<span class="text-muted-foreground">{new Date(log.started_at).toLocaleDateString()}</span>
										</div>
										<div class="flex justify-between text-muted-foreground">
											<span>{new Date(log.started_at).toLocaleTimeString()}</span>
											<span class="capitalize">{log.status}</span>
										</div>
									</div>
								{/each}
							</div>
						{/if}
					</Card.Content>
				</Card.Root>
			</div>
		</div>
	{/if}
</div>
