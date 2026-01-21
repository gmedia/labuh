<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Container, Play, Square, RotateCcw, Trash2, Plus } from '@lucide/svelte';
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Containers</h2>
			<p class="text-muted-foreground">Manage your running containers</p>
		</div>
		<Button class="gap-2">
			<Plus class="h-4 w-4" />
			Create Container
		</Button>
	</div>

	<Card.Root>
		<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
			<Container class="mb-4 h-12 w-12 text-muted-foreground/50" />
			<h3 class="text-lg font-semibold">No containers yet</h3>
			<p class="mb-4 text-sm text-muted-foreground">
				Create a container from an image to get started
			</p>
			<Button variant="outline">Pull an Image First</Button>
		</Card.Content>
	</Card.Root>
</div>
