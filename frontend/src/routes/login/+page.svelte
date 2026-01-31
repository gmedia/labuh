<script lang="ts">
	import { AuthController } from '$lib/features/auth/auth-controller.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';

	let ctrl = $state(new AuthController());
</script>

<div class="flex min-h-screen items-center justify-center bg-gradient-to-br from-background via-background to-muted/50 p-4">
	<Card.Root class="w-full max-w-md">
		<Card.Header class="text-center">
			<div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl overflow-hidden shadow-lg border">
				<img src="/logo.png" alt="Labuh Logo" class="h-12 w-12 object-contain" />
			</div>
			<Card.Title class="text-2xl font-bold">Welcome to Labuh</Card.Title>
			<Card.Description>Sign in to your account to continue</Card.Description>
		</Card.Header>
		<Card.Content>
			<form onsubmit={(e) => ctrl.login(e)} class="space-y-4">
				{#if ctrl.error}
					<div class="rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
						{ctrl.error}
					</div>
				{/if}

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

				<Button type="submit" class="w-full" disabled={ctrl.loading}>
					{ctrl.loading ? 'Signing in...' : 'Sign In'}
				</Button>
			</form>
		</Card.Content>
		<Card.Footer class="justify-center">
			<p class="text-sm text-muted-foreground">
				Don't have an account?
				<a href="/register" class="font-medium text-primary hover:underline">Sign up</a>
			</p>
		</Card.Footer>
	</Card.Root>
</div>
