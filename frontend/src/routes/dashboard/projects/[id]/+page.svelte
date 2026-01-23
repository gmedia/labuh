<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { api, type Project, type Domain } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import {
		FolderKanban, Play, Square, RotateCcw, Globe, Plus, Trash2, CheckCircle, AlertCircle,
		ArrowLeft, Lock
	} from '@lucide/svelte';

	let projectId = $derived($page.params.id ?? '');

	let project = $state<Project | null>(null);
	let domains = $state<Domain[]>([]);
	let loading = $state(true);
	let newDomain = $state('');
	let addingDomain = $state(false);
	let actionLoading = $state(false);

	async function loadProject() {
		if (!projectId) return;
		loading = true;
		const [projectResult, domainsResult] = await Promise.all([
			api.projects.get(projectId),
			api.projects.domains.list(projectId)
		]);

		if (projectResult.data) {
			project = projectResult.data;
		}
		if (domainsResult.data) {
			domains = domainsResult.data;
		}
		loading = false;
	}

	onMount(loadProject);

	async function addDomain() {
		if (!newDomain.trim() || !projectId) return;
		addingDomain = true;
		const result = await api.projects.domains.add(projectId, newDomain.trim());
		if (result.data) {
			domains = [...domains, result.data];
			newDomain = '';
		} else {
			alert(result.message || 'Failed to add domain');
		}
		addingDomain = false;
	}

	async function removeDomain(domain: string) {
		if (!projectId) return;
		if (!confirm(`Remove domain ${domain}?`)) return;
		await api.projects.domains.remove(projectId, domain);
		domains = domains.filter(d => d.domain !== domain);
	}

	async function deployProject() {
		if (!project) return;
		actionLoading = true;
		await api.projects.deploy(project.id);
		await loadProject();
		actionLoading = false;
	}

	async function stopProject() {
		if (!project) return;
		actionLoading = true;
		await api.projects.stop(project.id);
		await loadProject();
		actionLoading = false;
	}

	async function restartProject() {
		if (!project) return;
		actionLoading = true;
		await api.projects.restart(project.id);
		await loadProject();
		actionLoading = false;
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'running': return 'text-green-500';
			case 'stopped': return 'text-red-500';
			case 'deploying': return 'text-yellow-500';
			default: return 'text-muted-foreground';
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" href="/dashboard/projects">
			<ArrowLeft class="h-4 w-4" />
		</Button>
		<div class="flex-1">
			<h2 class="text-2xl font-bold tracking-tight">{project?.name || 'Loading...'}</h2>
			<p class="text-muted-foreground">{project?.description || 'Project details'}</p>
		</div>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if project}
		<div class="grid gap-6 md:grid-cols-2">
			<!-- Project Info Card -->
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2">
						<FolderKanban class="h-5 w-5" />
						Project Info
					</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="grid grid-cols-2 gap-4 text-sm">
						<div>
							<span class="text-muted-foreground">Status</span>
							<p class="{getStatusColor(project.status)} font-medium capitalize">{project.status}</p>
						</div>
						<div>
							<span class="text-muted-foreground">Image</span>
							<p class="font-mono text-xs">{project.image || 'Not set'}</p>
						</div>
						<div>
							<span class="text-muted-foreground">Port</span>
							<p>{project.port || 'Not set'}</p>
						</div>
						<div>
							<span class="text-muted-foreground">Slug</span>
							<p class="font-mono text-xs">{project.slug}</p>
						</div>
					</div>
					<div class="flex gap-2 pt-4 border-t">
						{#if project.status !== 'running'}
							<Button size="sm" onclick={deployProject} disabled={actionLoading}>
								<Play class="h-4 w-4 mr-1" /> Deploy
							</Button>
						{:else}
							<Button variant="outline" size="sm" onclick={stopProject} disabled={actionLoading}>
								<Square class="h-4 w-4 mr-1" /> Stop
							</Button>
							<Button variant="outline" size="sm" onclick={restartProject} disabled={actionLoading}>
								<RotateCcw class="h-4 w-4 mr-1" /> Restart
							</Button>
						{/if}
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Domains Card -->
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2">
						<Globe class="h-5 w-5" />
						Custom Domains
					</Card.Title>
					<Card.Description>
						Add custom domains with automatic SSL via Caddy
					</Card.Description>
				</Card.Header>
				<Card.Content class="space-y-4">
					<!-- Add Domain -->
					<div class="flex gap-2">
						<Input
							placeholder="example.com"
							bind:value={newDomain}
							onkeydown={(e) => e.key === 'Enter' && addDomain()}
						/>
						<Button onclick={addDomain} disabled={addingDomain || !newDomain.trim()}>
							<Plus class="h-4 w-4" />
						</Button>
					</div>

					<!-- Domain List -->
					{#if domains.length > 0}
						<div class="space-y-2">
							{#each domains as domain}
								<div class="flex items-center justify-between p-2 rounded-lg bg-muted/50">
									<div class="flex items-center gap-2">
										{#if domain.ssl_enabled}
											<Lock class="h-4 w-4 text-green-500" />
										{:else}
											<AlertCircle class="h-4 w-4 text-yellow-500" />
										{/if}
										<a
											href="https://{domain.domain}"
											target="_blank"
											class="text-sm hover:underline"
										>
											{domain.domain}
										</a>
										{#if domain.verified}
											<CheckCircle class="h-3 w-3 text-green-500" />
										{/if}
									</div>
									<Button
										variant="ghost"
										size="icon"
										class="h-8 w-8"
										onclick={() => removeDomain(domain.domain)}
									>
										<Trash2 class="h-4 w-4 text-destructive" />
									</Button>
								</div>
							{/each}
						</div>
					{:else}
						<p class="text-sm text-muted-foreground text-center py-4">
							No custom domains configured
						</p>
					{/if}
				</Card.Content>
			</Card.Root>
		</div>
	{:else}
		<Card.Root>
			<Card.Content class="py-12 text-center">
				<p class="text-muted-foreground">Project not found</p>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
