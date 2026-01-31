<script lang="ts">
	import { onMount } from 'svelte';
	import { ImageController } from '$lib/features/images/image-controller.svelte';
	import PullImageForm from '$lib/features/images/components/PullImageForm.svelte';
	import ImageList from '$lib/features/images/components/ImageList.svelte';

	let ctrl = $state(new ImageController());

	onMount(() => {
		ctrl.init();
	});
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Images</h2>
			<p class="text-muted-foreground">Pull and manage container images</p>
		</div>
	</div>

	<PullImageForm bind:ctrl />
	<ImageList bind:ctrl />
</div>
