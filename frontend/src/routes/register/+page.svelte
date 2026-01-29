<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Ship } from '@lucide/svelte';

	let name = $state('');
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let loading = $state(false);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';

		if (password !== confirmPassword) {
			error = 'Passwords do not match';
			return;
		}

		if (password.length < 6) {
			error = 'Password must be at least 6 characters';
			return;
		}

		loading = true;

		const result = await api.auth.register({ email, password, name: name || undefined });

		if (result.error) {
			error = result.message || 'Registration failed';
			loading = false;
			return;
		}

		goto('/dashboard');
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-gradient-to-br from-background via-background to-muted/50 p-4">
	<Card.Root class="w-full max-w-md">
		<Card.Header class="text-center">
			<div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl overflow-hidden shadow-lg border">
				<img src="/logo.png" alt="Labuh Logo" class="h-12 w-12 object-contain" />
			</div>
			<Card.Title class="text-2xl font-bold">Create an Account</Card.Title>
			<Card.Description>Get started with Labuh today</Card.Description>
		</Card.Header>
		<Card.Content>
			<form onsubmit={handleSubmit} class="space-y-4">
				{#if error}
					<div class="rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
						{error}
					</div>
				{/if}

				<div class="space-y-2">
					<Label for="name">Name (optional)</Label>
					<Input
						id="name"
						type="text"
						placeholder="Your name"
						bind:value={name}
					/>
				</div>

				<div class="space-y-2">
					<Label for="email">Email</Label>
					<Input
						id="email"
						type="email"
						placeholder="you@example.com"
						bind:value={email}
						required
					/>
				</div>

				<div class="space-y-2">
					<Label for="password">Password</Label>
					<Input
						id="password"
						type="password"
						placeholder="••••••••"
						bind:value={password}
						required
					/>
				</div>

				<div class="space-y-2">
					<Label for="confirmPassword">Confirm Password</Label>
					<Input
						id="confirmPassword"
						type="password"
						placeholder="••••••••"
						bind:value={confirmPassword}
						required
					/>
				</div>

				<Button type="submit" class="w-full" disabled={loading}>
					{loading ? 'Creating account...' : 'Create Account'}
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
