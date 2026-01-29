<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { api, type Container } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Container as ContainerIcon, Play, Square, RotateCcw, Trash2, Plus, X, Eye, EyeOff } from '@lucide/svelte';

	let containers = $state<Container[]>([]);
	let loading = $state(true);
	let showCreateDialog = $state(false);
	let newContainer = $state({
		name: '',
		image: '',
		envVars: [] as { key: string; value: string; masked: boolean }[],
		ports: [] as { hostPort: string; containerPort: string }[]
	});
	let creating = $state(false);
	let actionLoading = $state<string | null>(null);
	let envImportText = $state('');
	let showEnvImport = $state(false);

	async function loadContainers() {
		loading = true;
		const result = await api.containers.list(true);
		if (result.data) {
			containers = result.data;
		}
		loading = false;
	}

	onMount(loadContainers);

	function addEnvVar() {
		newContainer.envVars = [...newContainer.envVars, { key: '', value: '', masked: false }];
	}

	function removeEnvVar(index: number) {
		newContainer.envVars = newContainer.envVars.filter((_, i) => i !== index);
	}

	function addPort() {
		newContainer.ports = [...newContainer.ports, { hostPort: '', containerPort: '' }];
	}

	function removePort(index: number) {
		newContainer.ports = newContainer.ports.filter((_, i) => i !== index);
	}

	function importEnvFromText() {
		const lines = envImportText.split('\n').filter(line => line.trim() && !line.startsWith('#'));
		const newVars = lines.map(line => {
			const [key, ...valueParts] = line.split('=');
			return {
				key: key?.trim() || '',
				value: valueParts.join('=').trim().replace(/^["']|["']$/g, ''),
				masked: key?.toLowerCase().includes('secret') || key?.toLowerCase().includes('password') || key?.toLowerCase().includes('key')
			};
		}).filter(v => v.key);

		newContainer.envVars = [...newContainer.envVars, ...newVars];
		envImportText = '';
		showEnvImport = false;
	}

	async function loadImagePorts() {
		if (!newContainer.image) return;
		const result = await api.images.inspect(newContainer.image);
		if (result.data?.exposed_ports) {
			// Auto-add ports from image
			const newPorts = result.data.exposed_ports.map(port => ({
				hostPort: '',
				containerPort: port.replace('/tcp', '').replace('/udp', '')
			}));
			newContainer.ports = [...newContainer.ports, ...newPorts];
		}
	}

	async function createContainer() {
		if (!newContainer.name || !newContainer.image) return;
		creating = true;

		// Build env array (KEY=VALUE format)
		const env = newContainer.envVars
			.filter(e => e.key)
			.map(e => `${e.key}=${e.value}`);

		// Build ports object
		const ports: Record<string, string> = {};
		newContainer.ports
			.filter(p => p.containerPort && p.hostPort)
			.forEach(p => {
				ports[p.containerPort] = p.hostPort;
			});

		const result = await api.containers.create({
			name: newContainer.name,
			image: newContainer.image,
			env: env.length > 0 ? env : undefined,
			ports: Object.keys(ports).length > 0 ? ports : undefined,
		});

		if (result.data) {
			toast.success('Container created successfully');
			showCreateDialog = false;
			newContainer = { name: '', image: '', envVars: [], ports: [] };
			await loadContainers();
		} else {
			toast.error(result.error || 'Failed to create container');
		}
		creating = false;
	}

	async function startContainer(id: string) {
		actionLoading = id;
		const result = await api.containers.start(id);
		if (!result.error) toast.success('Container started');
		else toast.error(result.error);
		await loadContainers();
		actionLoading = null;
	}

	async function stopContainer(id: string) {
		actionLoading = id;
		const result = await api.containers.stop(id);
		if (!result.error) toast.success('Container stopped');
		else toast.error(result.error);
		await loadContainers();
		actionLoading = null;
	}

	async function restartContainer(id: string) {
		actionLoading = id;
		const result = await api.containers.restart(id);
		if (!result.error) toast.success('Container restarted');
		else toast.error(result.error);
		await loadContainers();
		actionLoading = null;
	}

	async function removeContainer(id: string) {
		if (!confirm('Are you sure you want to delete this container?')) return;
		actionLoading = id;
		const result = await api.containers.remove(id);
		if (!result.error) toast.success('Container removed');
		else toast.error(result.error);
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
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto py-8">
		<Card.Root class="w-full max-w-2xl mx-4">
			<Card.Header>
				<div class="flex items-center justify-between">
					<Card.Title>Create Container</Card.Title>
					<Button variant="ghost" size="icon" onclick={() => showCreateDialog = false}>
						<X class="h-4 w-4" />
					</Button>
				</div>
			</Card.Header>
			<Card.Content class="space-y-6 max-h-[60vh] overflow-y-auto">
				<!-- Basic Info -->
				<div class="grid gap-4 md:grid-cols-2">
					<div class="space-y-2">
						<Label for="name">Container Name</Label>
						<Input id="name" placeholder="my-container" bind:value={newContainer.name} />
					</div>
					<div class="space-y-2">
						<Label for="image">Image</Label>
						<div class="flex gap-2">
							<Input id="image" placeholder="nginx:latest" bind:value={newContainer.image} />
							<Button variant="outline" size="sm" onclick={loadImagePorts} disabled={!newContainer.image}>
								Auto-detect Ports
							</Button>
						</div>
					</div>
				</div>

				<!-- Environment Variables -->
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label>Environment Variables</Label>
						<div class="flex gap-2">
							<Button variant="outline" size="sm" onclick={() => showEnvImport = !showEnvImport}>
								Import .env
							</Button>
							<Button variant="outline" size="sm" onclick={addEnvVar}>
								<Plus class="h-3 w-3 mr-1" /> Add
							</Button>
						</div>
					</div>

					{#if showEnvImport}
						<div class="space-y-2 p-3 border rounded-lg bg-muted/50">
							<Label for="envImport">Paste .env content:</Label>
							<Textarea
								id="envImport"
								placeholder="KEY=value&#10;ANOTHER_KEY=another_value"
								bind:value={envImportText}
								rows={4}
							/>
							<div class="flex gap-2">
								<Button size="sm" onclick={importEnvFromText}>Import</Button>
								<Button variant="outline" size="sm" onclick={() => showEnvImport = false}>Cancel</Button>
							</div>
						</div>
					{/if}

					{#if newContainer.envVars.length > 0}
						<div class="space-y-2">
							{#each newContainer.envVars as envVar, index}
								<div class="flex gap-2 items-center">
									<Input
										placeholder="KEY"
										bind:value={envVar.key}
										class="w-1/3"
									/>
									<div class="relative flex-1">
										<Input
											placeholder="value"
											bind:value={envVar.value}
											type={envVar.masked ? 'password' : 'text'}
										/>
									</div>
									<Button variant="ghost" size="icon" onclick={() => envVar.masked = !envVar.masked}>
										{#if envVar.masked}
											<EyeOff class="h-4 w-4" />
										{:else}
											<Eye class="h-4 w-4" />
										{/if}
									</Button>
									<Button variant="ghost" size="icon" onclick={() => removeEnvVar(index)}>
										<X class="h-4 w-4 text-destructive" />
									</Button>
								</div>
							{/each}
						</div>
					{:else}
						<p class="text-sm text-muted-foreground">No environment variables configured.</p>
					{/if}
				</div>

				<!-- Port Mappings -->
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label>Port Mappings</Label>
						<Button variant="outline" size="sm" onclick={addPort}>
							<Plus class="h-3 w-3 mr-1" /> Add Port
						</Button>
					</div>

					{#if newContainer.ports.length > 0}
						<div class="space-y-2">
							{#each newContainer.ports as port, index}
								<div class="flex gap-2 items-center">
									<Input
										placeholder="Host Port (8080)"
										bind:value={port.hostPort}
										class="w-1/3"
									/>
									<span class="text-muted-foreground">:</span>
									<Input
										placeholder="Container Port (80)"
										bind:value={port.containerPort}
										class="w-1/3"
									/>
									<Button variant="ghost" size="icon" onclick={() => removePort(index)}>
										<X class="h-4 w-4 text-destructive" />
									</Button>
								</div>
							{/each}
						</div>
					{:else}
						<p class="text-sm text-muted-foreground">No port mappings configured.</p>
					{/if}
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
