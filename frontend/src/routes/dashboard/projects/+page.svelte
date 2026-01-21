<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { FolderKanban, Plus } from '@lucide/svelte';
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Projects</h2>
			<p class="text-muted-foreground">Manage your application deployments</p>
		</div>
		<Button class="gap-2">
			<Plus class="h-4 w-4" />
			Create Project
		</Button>
	</div>

	<Card.Root>
		<Card.Content class="flex flex-col items-center justify-center py-16 text-center">
			<FolderKanban class="mb-4 h-12 w-12 text-muted-foreground/50" />
			<h3 class="text-lg font-semibold">No projects yet</h3>
			<p class="mb-4 text-sm text-muted-foreground">
				Create a project to deploy and manage your applications
			</p>
			<Button>Create Your First Project</Button>
		</Card.Content>
	</Card.Root>
</div>
