<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { auth, theme } from '$lib/stores';

	let name = $state($auth.user?.name || '');
</script>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Settings</h2>
		<p class="text-muted-foreground">Manage your account and preferences</p>
	</div>

	<div class="grid gap-6 lg:grid-cols-2">
		<Card.Root>
			<Card.Header>
				<Card.Title>Profile</Card.Title>
				<Card.Description>Your account information</Card.Description>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="space-y-2">
					<Label for="email">Email</Label>
					<Input id="email" value={$auth.user?.email || ''} disabled />
				</div>
				<div class="space-y-2">
					<Label for="name">Name</Label>
					<Input id="name" bind:value={name} placeholder="Your name" />
				</div>
				<div class="space-y-2">
					<Label>Role</Label>
					<p class="text-sm text-muted-foreground capitalize">{$auth.user?.role}</p>
				</div>
			</Card.Content>
			<Card.Footer>
				<Button>Save Changes</Button>
			</Card.Footer>
		</Card.Root>

		<Card.Root>
			<Card.Header>
				<Card.Title>Appearance</Card.Title>
				<Card.Description>Customize the look and feel</Card.Description>
			</Card.Header>
			<Card.Content>
				<div class="flex items-center justify-between">
					<div>
						<p class="font-medium">Dark Mode</p>
						<p class="text-sm text-muted-foreground">Toggle between light and dark theme</p>
					</div>
					<Button variant="outline" onclick={() => theme.toggle()}>
						{$theme === 'dark' ? 'Switch to Light' : 'Switch to Dark'}
					</Button>
				</div>
			</Card.Content>
		</Card.Root>
	</div>
</div>
