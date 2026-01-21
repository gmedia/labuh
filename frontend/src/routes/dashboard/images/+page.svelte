<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Image, Download, Trash2 } from '@lucide/svelte';

	let imageUrl = $state('');
	let pulling = $state(false);
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
			<form class="flex gap-2">
				<Input
					placeholder="e.g., nginx:latest, ghcr.io/user/image:tag"
					bind:value={imageUrl}
					class="flex-1"
				/>
				<Button type="submit" disabled={pulling || !imageUrl} class="gap-2">
					<Download class="h-4 w-4" />
					{pulling ? 'Pulling...' : 'Pull'}
				</Button>
			</form>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Local Images</Card.Title>
		</Card.Header>
		<Card.Content class="flex flex-col items-center justify-center py-12 text-center">
			<Image class="mb-4 h-12 w-12 text-muted-foreground/50" />
			<h3 class="text-lg font-semibold">No images yet</h3>
			<p class="text-sm text-muted-foreground">
				Pull an image from a registry to get started
			</p>
		</Card.Content>
	</Card.Root>
</div>
