<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { browser } from '$app/environment';
	import { api, API_URL, type Stack, type Container, type Domain, type DeploymentLog, type StackHealth, type EnvVar, type ContainerResource, type ResourceMetric } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import {
		ArrowLeft, Play, Square, Trash2, RefreshCw, Terminal, Layers,
		Container as ContainerIcon, FileCode, Save, CheckCircle, XCircle,
		Globe, History, Webhook, Copy, AlertCircle, RotateCcw, Activity, Settings, Eye, EyeOff, Plus, ExternalLink,
		Cpu, HardDrive, Gauge, Users, Download, Upload, GitBranch
	} from '@lucide/svelte';
	import ResourceChart from '$lib/components/ResourceChart.svelte';
	import { activeTeam } from '$lib/stores';

	const stackId: string = $page.params.id ?? '';

	let stack = $state<Stack | null>(null);
	let containers = $state<Container[]>([]);
	let domains = $state<Domain[]>([]);
	let deployments = $state<DeploymentLog[]>([]);
	let logs = $state<Map<string, string[]>>(new Map());

	// New state for health and env vars
	let health = $state<StackHealth | null>(null);
	let envVars = $state<EnvVar[]>([]);
	let newEnvKey = $state('');
	let newEnvValue = $state('');
	let newEnvContainer = $state(''); // empty for global
	let newEnvSecret = $state(false);
	let showSecrets = $state<Set<string>>(new Set());

	// New Resource & Metrics state
	let resourceLimits = $state<ContainerResource[]>([]);
	let metrics = $state<ResourceMetric[]>([]);
	let selectedRange = $state("1h");
	let savingResources = $state<Set<string>>(new Set());

	let cronSchedule = $state('');
	let healthCheckPath = $state('');
	let healthCheckInterval = $state(30);
	let savingAutomation = $state(false);

	$effect(() => {
		if (stack) {
			cronSchedule = stack.cron_schedule || '';
			healthCheckPath = stack.health_check_path || '';
			healthCheckInterval = stack.health_check_interval || 30;
		}
	});

	let loading = $state(true);
	let actionLoading = $state(false);

	let showComposeEditor = $state(false);
	let editedCompose = $state('');
	let savingCompose = $state(false);

	let selectedContainerLogs = $state<string | null>(null);
	let newDomain = $state('');
	let newDomainContainer = $state('');
	let newDomainPort = $state(80);
	let addingDomain = $state(false);

	const isViewer = $derived($activeTeam?.role === 'Viewer');

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

	async function loadHealth() {
		const result = await api.stacks.health(stackId);
		if (result.data) {
			health = result.data;
		}
	}

	async function loadEnvVars() {
		const result = await api.stacks.env.list(stackId);
		if (result.data) {
			envVars = result.data;
		}
	}

	async function loadResourceLimits() {
		const result = await api.stacks.resources.getLimits(stackId);
		if (result.data) {
			let limits = result.data;

			// Ensure all services have an entry
			const serviceNames = containers.map(c => c.labels?.['labuh.service.name'] || c.names[0]?.replace(/^\//, ''));
			for (const name of serviceNames) {
				if (!limits.find(l => l.service_name === name)) {
					limits.push({
						id: '',
						stack_id: stackId,
						service_name: name,
						cpu_limit: undefined,
						memory_limit: undefined,
						created_at: '',
						updated_at: ''
					});
				}
			}
			resourceLimits = limits;
		}
	}

	async function loadMetrics() {
		const result = await api.stacks.resources.getMetrics(stackId, selectedRange);
		if (result.data) {
			metrics = result.data;
		}
	}

	async function updateResourceLimit(serviceName: string, cpuLimit: number | undefined, memoryLimit: number | undefined) {
		savingResources.add(serviceName);

		// Memory limit input is usually in MB from UI, convert to bytes for API
		const memBytes = memoryLimit ? memoryLimit * 1024 * 1024 : undefined;

		const result = await api.stacks.resources.updateLimits(stackId, serviceName, {
			cpu_limit: cpuLimit,
			memory_limit: memBytes
		});

		if (result.error) {
			toast.error(`Error: ${result.error}`);
		} else {
			toast.success(`Limits updated for ${serviceName}. Redeploy to apply.`);
			await loadResourceLimits();
		}
		savingResources.delete(serviceName);
	}

	async function loadContainerLogs(containerId: string) {
		selectedContainerLogs = containerId;
		const result = await api.containers.logs(containerId, 100);
		if (result.data) {
			logs.set(containerId, result.data);
			logs = new Map(logs);
		}
	}

	onMount(() => {
		(async () => {
			await loadStack();
			if (stack) {
				await Promise.all([
					loadContainers(),
					loadDomains(),
					loadDeployments(),
					loadHealth(),
					loadEnvVars(),
					loadResourceLimits(),
					loadMetrics()
				]);
			}
		})();

		const interval = setInterval(() => {
			loadContainers();
			loadHealth();
			loadMetrics();
		}, 30000); // 30s auto-refresh

		return () => clearInterval(interval);
	});

	$effect(() => {
		if (selectedRange) {
			loadMetrics();
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

	async function redeployStack(serviceName?: string) {
		if (!stack) return;
		const msg = serviceName ? `Recreate container ${serviceName}?` : 'Recreate all containers in this stack? This will apply any environment variable changes.';
		if (!confirm(msg)) return;
		actionLoading = true;
		await api.stacks.redeploy(stack.id, serviceName);
		await Promise.all([loadStack(), loadContainers(), loadHealth()]);
		actionLoading = false;
	}

	async function removeStack() {
		if (!stack) return;
		if (!confirm('Are you sure you want to delete this stack and all its containers?')) return;
		actionLoading = true;
		await api.stacks.remove(stack.id);
		goto('/dashboard/stacks');
	}

	async function rollbackStack() {
		if (!stack) return;
		if (!confirm('Revert all containers in this stack to the last stable images?')) return;
		actionLoading = true;
		try {
			const result = await api.stacks.rollback(stack.id);
			if (result.error) {
				toast.error(result.message || result.error);
			} else {
				toast.success('Rollback triggered');
				await Promise.all([loadStack(), loadContainers(), loadHealth()]);
			}
		} catch (e: any) {
			toast.error(e.message || 'Failed to rollback stack');
		} finally {
			actionLoading = false;
		}
	}

	async function updateAutomation() {
		if (!stack) return;
		savingAutomation = true;
		try {
			const result = await api.stacks.updateAutomation(stack.id, {
				cron_schedule: cronSchedule,
				health_check_path: healthCheckPath,
				health_check_interval: healthCheckInterval
			});
			if (result.error) {
				toast.error(result.message || result.error);
			} else {
				toast.success('Automation settings updated');
				await loadStack();
			}
		} catch (e: any) {
			toast.error(e.message || 'Failed to update automation settings');
		} finally {
			savingAutomation = false;
		}
	}

	async function saveCompose() {
		if (!stack) return;
		savingCompose = true;
		try {
			const result = await api.stacks.updateCompose(stack.id, editedCompose);
			if (result.error) {
				toast.error(result.message || result.error);
			} else {
				toast.success('Stack updated and redeployment triggered');
				showComposeEditor = false;
				await Promise.all([loadStack(), loadContainers(), loadHealth()]);
			}
		} catch (e: any) {
			toast.error(e.message || 'Failed to update stack');
		} finally {
			savingCompose = false;
		}
	}

	async function exportStack() {
		if (!stack) return;
		actionLoading = true;
		try {
			const result = await api.stacks.backup(stack.id);
			if (result.data) {
				const blob = new Blob([JSON.stringify(result.data, null, 2)], { type: 'application/json' });
				const url = URL.createObjectURL(blob);
				const a = document.createElement('a');
				a.href = url;
				a.download = `labuh-backup-${stack.name}-${new Date().toISOString().split('T')[0]}.json`;
				document.body.appendChild(a);
				a.click();
				document.body.removeChild(a);
				URL.revokeObjectURL(url);
				toast.success('Backup downloaded');
			} else {
				toast.error(result.message || result.error || 'Failed to export stack');
			}
		} catch (e: any) {
			toast.error(e.message || 'Failed to export stack');
		} finally {
			actionLoading = false;
		}
	}

	async function syncGit() {
		if (!stack) return;
		actionLoading = true;
		try {
			const result = await api.stacks.syncGit(stack.id);
			if (result.data) {
				toast.success('Stack synced with Git and redeployed');
				await Promise.all([loadStack(), loadContainers(), loadHealth()]);
			} else {
				toast.error(result.message || result.error || 'Failed to sync Git');
			}
		} catch (e: any) {
			toast.error(e.message || 'Failed to sync Git');
		} finally {
			actionLoading = false;
		}
	}

	async function addDomain() {
		if (!newDomain || !newDomainContainer || !stack) return;
		addingDomain = true;
		await api.stacks.domains.add(stack.id, {
			domain: newDomain,
			container_name: newDomainContainer,
			container_port: newDomainPort
		});
		newDomain = '';
		newDomainContainer = '';
		newDomainPort = 80;
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

	async function addEnvVar() {
		if (!stack || !newEnvKey) return;
		await api.stacks.env.set(stack.id, {
			container_name: newEnvContainer,
			key: newEnvKey,
			value: newEnvValue,
			is_secret: newEnvSecret
		});
		newEnvKey = '';
		newEnvValue = '';
		newEnvContainer = '';
		newEnvSecret = false;
		await loadEnvVars();
	}

	async function deleteEnvVar(key: string, containerName: string) {
		if (!stack || !confirm(`Delete environment variable "${key}" for container "${containerName || 'Global'}"?`)) return;
		await api.stacks.env.delete(stack.id, key, containerName);
		await loadEnvVars();
	}

	function toggleSecretVisibility(id: string) {
		const newSet = new Set(showSecrets);
		if (newSet.has(id)) {
			newSet.delete(id);
		} else {
			newSet.add(id);
		}
		showSecrets = newSet;
	}

	async function regenerateWebhook() {
		if (!stack || !confirm('Regenerate webhook token? Previous URL will stop working.')) return;
		const result = await api.stacks.regenerateWebhookToken(stack.id);
		if (result.data && stack) {
			stack.webhook_token = result.data.token;
		}
	}

	function copyToClipboard(text: string) {
		if (navigator.clipboard) {
			navigator.clipboard.writeText(text).then(() => {
				toast.success('Copied to clipboard');
			}).catch(() => {
				fallbackCopyTextToClipboard(text);
			});
		} else {
			fallbackCopyTextToClipboard(text);
		}
	}

	function fallbackCopyTextToClipboard(text: string) {
		const textArea = document.createElement("textarea");
		textArea.value = text;

		// Avoid scrolling to bottom
		textArea.style.top = "0";
		textArea.style.left = "0";
		textArea.style.position = "fixed";

		document.body.appendChild(textArea);
		textArea.focus();
		textArea.select();

		try {
			const successful = document.execCommand('copy');
			if (successful) {
				toast.success('Copied to clipboard');
			} else {
				toast.error('Failed to copy');
			}
		} catch (err) {
			toast.error('Failed to copy');
		}

		document.body.removeChild(textArea);
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
	let selectedWebhookService = $state('');
	const webhookUrl = $derived(stack?.webhook_token ? `${browser ? window.location.origin : API_URL}/api/webhooks/deploy/${stack.id}/${stack.webhook_token}` : '');
	const filteredWebhookUrl = $derived(selectedWebhookService ? `${webhookUrl}?service=${selectedWebhookService}` : webhookUrl);
</script>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" onclick={() => goto('/dashboard/stacks')}>
			<ArrowLeft class="h-5 w-5" />
		</Button>
		<div class="flex-1">
			<div class="flex items-center gap-2">
				<h2 class="text-2xl font-bold tracking-tight">{stack?.name || 'Loading...'}</h2>
				{#if $activeTeam?.team}
					<span class="text-[10px] uppercase font-bold text-muted-foreground bg-muted-foreground/10 px-2 py-0.5 rounded flex items-center gap-1">
						<Users class="h-2.5 w-2.5" />
						{$activeTeam.team.name}
					</span>
				{/if}
			</div>
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
								{#if !isViewer}
									{#if stack.status !== 'running'}
										<Button variant="outline" size="sm" onclick={startStack} disabled={actionLoading}>
											<Play class="h-4 w-4 mr-1" /> Start
										</Button>
									{:else}
										<Button variant="outline" size="sm" onclick={stopStack} disabled={actionLoading}>
											<Square class="h-4 w-4 mr-1" /> Stop
										</Button>
									{/if}
									<Button
										variant="outline"
										size="sm"
										onclick={rollbackStack}
										disabled={actionLoading || !stack.last_stable_images}
										title="Rollback to last stable version"
									>
										<RotateCcw class="h-4 w-4 mr-1 {actionLoading ? 'animate-spin' : ''}" />
										{actionLoading ? 'Rolling back...' : 'Rollback'}
									</Button>
									<Button variant="outline" size="sm" onclick={() => redeployStack()} disabled={actionLoading} title="Recreate containers to apply changes">
										<RotateCcw class="h-4 w-4 mr-1" /> Restart
									</Button>
									<Button variant="outline" size="sm" onclick={() => showComposeEditor = !showComposeEditor}>
										<FileCode class="h-4 w-4 mr-1" /> Edit
									</Button>
									{#if stack.git_url}
										<Button variant="outline" size="sm" onclick={syncGit} disabled={actionLoading} title="Sync with Git and redeploy">
											<GitBranch class="h-4 w-4 mr-1 {actionLoading ? 'animate-spin' : ''}" /> Sync
										</Button>
									{/if}
									<Button variant="outline" size="sm" onclick={exportStack} disabled={actionLoading} title="Export stack as JSON backup">
										<Download class="h-4 w-4 mr-1" /> Export
									</Button>
									<Button variant="outline" size="sm" onclick={removeStack} disabled={actionLoading}>
										<Trash2 class="h-4 w-4 text-destructive" />
									</Button>
								{:else}
									<span class="text-xs text-muted-foreground italic px-2">Read Only Mode</span>
								{/if}
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
										{#if !isViewer}
											<Button variant="ghost" size="sm" onclick={() => loadContainerLogs(container.id)} title="View Logs">
												<Terminal class="h-4 w-4" />
											</Button>
											<Button
												variant="ghost"
												size="sm"
												onclick={() => redeployStack(container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, ''))}
												title="Redeploy Service"
												disabled={actionLoading}
											>
												<RefreshCw class="h-4 w-4 {actionLoading ? 'animate-spin' : ''}" />
											</Button>
										{:else}
											<Button variant="ghost" size="sm" onclick={() => loadContainerLogs(container.id)} title="View Logs">
												<Terminal class="h-4 w-4" />
											</Button>
										{/if}
									</div>
								</div>
							{/each}
						{/if}
					</Card.Content>
				</Card.Root>

				<!-- Monitoring -->
				<Card.Root>
					<Card.Header>
						<div class="flex items-center justify-between">
							<Card.Title class="flex items-center gap-2">
								<Activity class="h-5 w-5" />
								Monitoring
							</Card.Title>
							<select bind:value={selectedRange} class="bg-background border rounded px-2 py-1 text-xs">
								<option value="1h">Last Hour</option>
								<option value="6h">Last 6 Hours</option>
								<option value="24h">Last 24 Hours</option>
								<option value="7d">Last 7 Days</option>
							</select>
						</div>
					</Card.Header>
					<Card.Content class="space-y-6">
						{#if metrics.length === 0}
							<p class="text-sm text-muted-foreground text-center py-8">No monitoring data available yet.</p>
						{:else}
							<div class="space-y-4">
								<div>
									<h4 class="text-xs font-medium uppercase text-muted-foreground mb-1">CPU Usage (%)</h4>
									<ResourceChart {metrics} type="cpu" />
								</div>
								<div>
									<h4 class="text-xs font-medium uppercase text-muted-foreground mb-1">Memory Usage (MB)</h4>
									<ResourceChart {metrics} type="memory" />
								</div>
							</div>
						{/if}
					</Card.Content>
				</Card.Root>

				<!-- Resource Limits -->
				<Card.Root>
					<Card.Header>
						<Card.Title class="flex items-center gap-2">
							<Settings class="h-5 w-5" />
							Resource Limits
						</Card.Title>
						<Card.Description>Configure CPU and Memory constraints per service</Card.Description>
					</Card.Header>
					<Card.Content class="space-y-4">
						{#each containers as container}
							{@const serviceName = container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}
							{@const limits = resourceLimits.find(l => l.service_name === serviceName)}
							<div class="p-4 border rounded-lg space-y-3">
								<div class="flex items-center justify-between">
									<h4 class="font-medium text-sm">{serviceName}</h4>
									<Button
										size="sm"
										variant="outline"
										onclick={() => updateResourceLimit(serviceName, limits?.cpu_limit, limits?.memory_limit ? limits.memory_limit / (1024 * 1024) : undefined)}
										disabled={savingResources.has(serviceName) || isViewer}
									>
										{savingResources.has(serviceName) ? 'Saving...' : 'Save'}
									</Button>
								</div>
								<div class="grid grid-cols-2 gap-4">
									<div class="space-y-1">
										<Label class="text-[10px] uppercase font-bold text-muted-foreground">CPU Limit (Cores)</Label>
										<Input
											type="number"
											step="0.1"
											placeholder="e.g. 0.5"
											bind:value={() => resourceLimits.find(l => l.service_name === serviceName)?.cpu_limit, (v) => {
												let l = resourceLimits.find(l => l.service_name === serviceName);
												if (l) l.cpu_limit = v;
												else resourceLimits.push({ id: '', stack_id: stackId, service_name: serviceName, cpu_limit: v, memory_limit: undefined, created_at: '', updated_at: '' });
											}}
										/>
									</div>
									<div class="space-y-1">
										<Label class="text-[10px] uppercase font-bold text-muted-foreground">Memory Limit (MB)</Label>
										<Input
											type="number"
											placeholder="e.g. 512"
											bind:value={() => {
												const limit = resourceLimits.find(l => l.service_name === serviceName);
												return limit?.memory_limit ? limit.memory_limit / (1024 * 1024) : undefined;
											}, (v) => {
												let l = resourceLimits.find(l => l.service_name === serviceName);
												const bytes = v ? v * 1024 * 1024 : undefined;
												if (l) l.memory_limit = bytes;
												else resourceLimits.push({ id: '', stack_id: stackId, service_name: serviceName, cpu_limit: undefined, memory_limit: bytes, created_at: '', updated_at: '' });
											}}
										/>
									</div>
								</div>
							</div>
						{/each}
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

			<!-- Sidebar: Health, Env Vars, Domains & Webhooks & History -->
			<div class="space-y-6">
				<!-- Health Overview -->
				<Card.Root>
					<Card.Header>
						<div class="flex items-center justify-between">
							<Card.Title class="flex items-center gap-2">
								<Activity class="h-5 w-5" />
								Health
							</Card.Title>
							<Button variant="ghost" size="sm" onclick={loadHealth}>
								<RefreshCw class="h-3 w-3" />
							</Button>
						</div>
					</Card.Header>
					<Card.Content>
						{#if health}
							<div class="space-y-3">
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Status</span>
									<span class="font-medium capitalize {health.status === 'healthy' ? 'text-green-500' : health.status === 'partial' ? 'text-yellow-500' : 'text-red-500'}">
										{health.status}
									</span>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Containers</span>
									<span class="font-medium">{health.healthy_count}/{health.total_count} running</span>
								</div>
								{#if health.containers.length > 0}
									<div class="space-y-1 pt-2 border-t">
										{#each health.containers as c}
											<div class="flex items-center justify-between text-xs">
												<span class="truncate flex-1">{c.name}</span>
												<span class="ml-2 {c.state === 'running' ? 'text-green-500' : 'text-red-500'}">{c.state}</span>
											</div>
										{/each}
									</div>
								{/if}
							</div>
						{:else}
							<p class="text-sm text-muted-foreground text-center py-2">Loading...</p>
						{/if}
					</Card.Content>
				</Card.Root>

				<!-- Environment Variables -->
				<Card.Root>
					<Card.Header>
						<Card.Title class="flex items-center gap-2">
							<Settings class="h-5 w-5" />
							Environment Variables
						</Card.Title>
						<Card.Description>Configure stack environment</Card.Description>
					</Card.Header>
					<Card.Content class="space-y-4">
						<div class="space-y-2">
							<div class="flex gap-2">
								<Input placeholder="KEY" bind:value={newEnvKey} class="flex-1 font-mono text-xs" />
								<Input placeholder="value" bind:value={newEnvValue} class="flex-1 font-mono text-xs" />
							</div>
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-3">
									<label class="flex items-center gap-2 text-xs cursor-pointer">
										<input type="checkbox" bind:checked={newEnvSecret} class="rounded" />
										Secret
									</label>

									<select
										bind:value={newEnvContainer}
										class="h-8 rounded-md border border-input bg-background px-2 text-[10px]"
									>
										<option value="">Global</option>
										{#each containers as container}
											<option value={container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}>
												{container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}
											</option>
										{/each}
									</select>
								</div>
								<Button size="sm" variant="outline" onclick={addEnvVar} disabled={isViewer}>
									<Plus class="h-3 w-3 mr-1" /> Add
								</Button>
							</div>
						</div>

						<div class="space-y-1 max-h-60 overflow-y-auto">
							{#if envVars.length === 0}
								<p class="text-xs text-muted-foreground text-center py-2">No environment variables</p>
							{:else}
								{#each envVars as env}
									<div class="flex items-center justify-between p-2 rounded border bg-background/50 text-xs gap-2">
										<div class="flex-1 min-w-0">
											<div class="flex items-center gap-1.5 mb-0.5">
												<span class="font-mono font-medium truncate">{env.key}</span>
												{#if env.container_name}
													<span class="px-1.5 py-0.5 rounded-full bg-primary/10 text-primary text-[9px]">
														{env.container_name}
													</span>
												{:else}
													<span class="px-1.5 py-0.5 rounded-full bg-muted text-muted-foreground text-[9px]">
														Global
													</span>
												{/if}
											</div>
											{#if env.is_secret && !showSecrets.has(env.id)}
												<span class="text-muted-foreground">********</span>
											{:else}
												<span class="font-mono text-muted-foreground truncate">{env.value}</span>
											{/if}
										</div>
										<div class="flex items-center gap-1 ml-2">
											{#if env.is_secret}
												<Button variant="ghost" size="icon" class="h-5 w-5" onclick={() => toggleSecretVisibility(env.id)} title="Toggle visibility">
													{#if showSecrets.has(env.id)}
														<EyeOff class="h-3 w-3" />
													{:else}
														<Eye class="h-3 w-3" />
													{/if}
												</Button>
											{/if}
											{#if !isViewer}
												<Button variant="ghost" size="icon" class="h-5 w-5 text-destructive" onclick={() => deleteEnvVar(env.key, env.container_name)} title="Delete">
													<Trash2 class="h-3 w-3" />
												</Button>
											{/if}
										</div>
									</div>
								{/each}
							{/if}
						</div>
						<p class="text-xs text-muted-foreground">
							Changes apply after stack restart.
						</p>
					</Card.Content>
				</Card.Root>

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
							<div class="space-y-4">
								<div class="space-y-2">
									<div class="flex items-center justify-between">
										<Label>Webhook URL</Label>
										<select
											bind:value={selectedWebhookService}
											class="h-7 rounded-md border border-input bg-background px-2 text-[10px]"
										>
											<option value="">All Services</option>
											{#each containers as container}
												<option value={container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}>
													Only {container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}
												</option>
											{/each}
										</select>
									</div>
									<div class="flex items-center gap-2">
										<Input readonly value={filteredWebhookUrl} class="font-mono text-[10px]" />
										<Button variant="outline" size="icon" class="h-9 w-9" onclick={() => copyToClipboard(filteredWebhookUrl)}>
											<Copy class="h-4 w-4" />
										</Button>
									</div>
									<p class="text-[10px] text-muted-foreground">
										{#if selectedWebhookService}
											POST to this URL to pull latest image and redeploy <strong>{selectedWebhookService}</strong> only.
										{:else}
											POST to this URL to pull latest images and redeploy all containers in this stack.
										{/if}
									</p>
								</div>
								<Button variant="outline" size="sm" class="w-full text-xs" onclick={regenerateWebhook} disabled={isViewer}>
									Regenerate Token
								</Button>
							</div>
						{:else}
							<Button class="w-full" onclick={regenerateWebhook}>
								Generate Webhook
							</Button>
						{/if}
					</Card.Content>
				</Card.Root>

				<!-- Automation -->
				<Card.Root>
					<Card.Header>
						<Card.Title class="flex items-center gap-2">
							<Play class="h-5 w-5" />
							Automation
						</Card.Title>
						<Card.Description>Scheduled pulls & health checks</Card.Description>
					</Card.Header>
					<Card.Content class="space-y-4">
						<div class="space-y-2">
							<Label class="text-xs">Cron Schedule (e.g. 0 0 * * * *)</Label>
							<Input bind:value={cronSchedule} placeholder="0 0 * * * *" class="font-mono text-xs" />
							<p class="text-[10px] text-muted-foreground italic">Standard cron: sec min hour day month dow</p>
						</div>
						<div class="space-y-2">
							<Label class="text-xs">Health Check Path (HTTP URL)</Label>
							<Input bind:value={healthCheckPath} placeholder="http://yourapp.labuh:8080/health" class="font-mono text-xs" />
						</div>
						<div class="space-y-2">
							<Label class="text-xs">Interval (seconds)</Label>
							<Input type="number" bind:value={healthCheckInterval} class="font-mono text-xs" />
						</div>
						<Button variant="outline" size="sm" class="w-full" onclick={updateAutomation} disabled={savingAutomation || isViewer}>
							<Save class="h-3 w-3 mr-1" /> {savingAutomation ? 'Saving...' : 'Save Automation'}
						</Button>
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
						<div class="grid gap-2">
							<div class="flex gap-2">
								<Input placeholder="example.com" bind:value={newDomain} class="flex-1" />
							</div>
							<div class="flex gap-2">
								<select
									bind:value={newDomainContainer}
									class="flex-1 h-9 rounded-md border border-input bg-background px-3 text-sm"
								>
									<option value="">Select container...</option>
									{#each containers as container}
										<option value={container.names[0]?.replace(/^\//, '') || container.id}>
											{container.names[0]?.replace(/^\//, '') || container.id.substring(0, 12)}
										</option>
									{/each}
								</select>
								<Input
									type="number"
									placeholder="Port"
									bind:value={newDomainPort}
									class="w-20"
								/>
								<Button size="icon" onclick={addDomain} disabled={addingDomain || !newDomain || !newDomainContainer || isViewer}>
									{#if addingDomain}
										<div class="animate-spin rounded-full h-3 w-3 border-b-2 border-primary-foreground"></div>
									{:else}
										<div class="h-4 w-4">+</div>
									{/if}
								</Button>
							</div>
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
									<span class="text-xs text-muted-foreground">→ {domain.container_name}:{domain.container_port}</span>
										</div>
										<div class="flex gap-1">
											{#if !isViewer}
												{#if !domain.verified}
													<Button variant="ghost" size="icon" class="h-6 w-6" onclick={() => verifyDomain(domain.domain)} title="Verify">
														<CheckCircle class="h-3 w-3" />
													</Button>
												{/if}
												<Button variant="ghost" size="icon" class="h-6 w-6 text-destructive" onclick={() => removeDomain(domain.domain)} title="Remove">
													<Trash2 class="h-3 w-3" />
												</Button>
											{:else}
												<span class="text-[10px] text-muted-foreground mr-1">View Only</span>
											{/if}
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
