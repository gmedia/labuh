<script lang="ts">
	import { onMount } from 'svelte';
	import { activeTeam } from '$lib/stores';
	import { SettingsController } from '$lib/features/settings/settings-controller.svelte';
	import ProfileSettings from '$lib/features/settings/components/ProfileSettings.svelte';
	import AppearanceSettings from '$lib/features/settings/components/AppearanceSettings.svelte';
	import RegistrySettings from '$lib/features/settings/components/RegistrySettings.svelte';

	let ctrl = $state(new SettingsController());

	onMount(() => {
		ctrl.init();
	});

	$effect(() => {
		if ($activeTeam?.team) {
			ctrl.loadRegistries();
		}
	});
</script>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Settings</h2>
		<p class="text-muted-foreground">Manage your account and preferences</p>
	</div>

	<div class="grid gap-6 lg:grid-cols-2">
		<ProfileSettings bind:ctrl />
		<AppearanceSettings />
		<RegistrySettings bind:ctrl />
	</div>
</div>
