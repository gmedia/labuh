<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { api, type Image } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Image as ImageIcon, Download, Trash2 } from '@lucide/svelte';

	let images = $state<Image[]>([]);
	let loading = $state(true);
	let imageUrl = $state('');
	let pulling = $state(false);
	let actionLoading = $state<string | null>(null);

	async function loadImages() {
		loading = true;
		const result = await api.images.list();
		if (result.data) {
			images = result.data;
		}
		loading = false;
	}

	onMount(loadImages);

	async function pullImage() {
		if (!imageUrl) return;
		pulling = true;
		const result = await api.images.pull(imageUrl);
		if (result.data) {
			toast.success(`Image ${imageUrl} pulled successfully`);
			imageUrl = '';
			await loadImages();
		} else {
			toast.error(result.error || 'Failed to pull image');
		}
		pulling = false;
	}

	async function removeImage(id: string) {
		if (!confirm('Are you sure you want to delete this image?')) return;
		actionLoading = id;
		const result = await api.images.remove(id);
		if (result.error) {
			toast.error(result.error || 'Failed to remove image');
		} else {
			toast.success('Image removed');
			await loadImages();
		}
		actionLoading = null;
	}

	function formatSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	function formatDate(timestamp: number): string {
		return new Date(timestamp * 1000).toLocaleDateString();
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Images</h2>
			<p class="text-muted-foreground">Pull and manage container images</p>
		</div>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title>Pull Image</Card.Title>
			<Card.Description>Pull a container image from Docker Hub or a private registry</Card.Description>
		</Card.Header>
		<Card.Content>
			<form onsubmit={(e) => { e.preventDefault(); pullImage(); }} class="flex gap-2">
				<Input
					placeholder="e.g., nginx:latest, ghcr.io/user/image:tag"
					bind:value={imageUrl}
					class="flex-1"
					disabled={pulling}
				/>
				<Button type="submit" disabled={pulling || !imageUrl} class="gap-2">
					<Download class="h-4 w-4" />
					{pulling ? 'Pulling...' : 'Pull'}
				</Button>
			</form>
			{#if pulling}
				<p class="mt-2 text-sm text-muted-foreground">Pulling image... This may take a while.</p>
			{/if}
		</Card.Content>
	</Card.Root>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if images.length === 0}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-12 text-center">
				<ImageIcon class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<h3 class="text-lg font-semibold">No images yet</h3>
				<p class="text-sm text-muted-foreground">
					Pull an image from a registry to get started
				</p>
			</Card.Content>
		</Card.Root>
	{:else}
		<Card.Root>
			<Card.Header>
				<Card.Title>Local Images</Card.Title>
			</Card.Header>
			<Card.Content>
				<div class="space-y-3">
					{#each images as image}
						<div class="flex items-center justify-between rounded-lg border p-3">
							<div class="flex items-center gap-3">
								<ImageIcon class="h-6 w-6 text-muted-foreground" />
								<div>
									<p class="font-medium">{image.repo_tags[0] || image.id.slice(7, 19)}</p>
									<p class="text-xs text-muted-foreground">
										{formatSize(image.size)} · Created {formatDate(image.created)}
									</p>
								</div>
							</div>
							<Button
								variant="outline"
								size="icon"
								onclick={() => removeImage(image.id)}
								disabled={actionLoading === image.id}
							>
								<Trash2 class="h-4 w-4 text-destructive" />
							</Button>
						</div>
					{/each}
				</div>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
