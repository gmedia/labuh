<script lang="ts">
	import { AuthController } from '$lib/features/auth/auth-controller.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';

	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let ctrl = $state(new AuthController());

	onMount(async () => {
		await ctrl.checkSetup();
		if (ctrl.setupRequired === false) {
			goto('/login');
		}
	});
</script>

<div class="flex min-h-screen items-center justify-center bg-gradient-to-br from-background via-background to-muted/50 p-4">
	<Card.Root class="w-full max-w-md">
		<Card.Header class="text-center">
			<div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl overflow-hidden shadow-lg border">
				<img src="/logo.png" alt="Labuh Logo" class="h-12 w-12 object-contain" />
			</div>
			<Card.Title class="text-2xl font-bold">
				{ctrl.setupRequired ? 'Setup Administrator' : 'Create an Account'}
			</Card.Title>
			<Card.Description>
				{ctrl.setupRequired
					? 'Configure the first administrator account for Labuh'
					: 'Get started with Labuh today'}
			</Card.Description>
		</Card.Header>
		<Card.Content>
			<form onsubmit={(e) => ctrl.register(e)} class="space-y-4">
				{#if ctrl.error}
					<div class="rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
						{ctrl.error}
					</div>
				{/if}

				<div class="space-y-2">
					<Label for="name">Name (optional)</Label>
					<Input
						id="name"
						type="text"
						placeholder="Your name"
						bind:value={ctrl.name}
					/>
				</div>

				<div class="space-y-2">
					<Label for="email">Email</Label>
					<Input
						id="email"
						type="email"
						placeholder="you@example.com"
						bind:value={ctrl.email}
						required
					/>
				</div>

				<div class="space-y-2">
					<Label for="password">Password</Label>
					<Input
						id="password"
						type="password"
						placeholder="••••••••"
						bind:value={ctrl.password}
						required
					/>
				</div>

				<div class="space-y-2">
					<Label for="confirmPassword">Confirm Password</Label>
					<Input
						id="confirmPassword"
						type="password"
						placeholder="••••••••"
						bind:value={ctrl.confirmPassword}
						required
					/>
				</div>

				<Button type="submit" class="w-full" disabled={ctrl.loading}>
					{ctrl.loading ? 'Creating account...' : (ctrl.setupRequired ? 'Complete Setup' : 'Create Account')}
				</Button>
			</form>
		</Card.Content>
		<Card.Footer class="justify-center">
			<p class="text-sm text-muted-foreground">
				Already have an account?
				<a href="/login" class="font-medium text-primary hover:underline">Sign in</a>
			</p>
		</Card.Footer>
	</Card.Root>
</div>
