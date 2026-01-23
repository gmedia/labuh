<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Ship, ArrowRight, Container, Zap, Shield } from '@lucide/svelte';
</script>

<div class="flex min-h-screen flex-col">
	<!-- Header -->
	<header class="flex items-center justify-between border-b px-6 py-4">
		<div class="flex items-center gap-2">
			<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary text-primary-foreground">
				<Ship class="h-6 w-6" />
			</div>
			<span class="text-xl font-bold">Labuh</span>
		</div>
		<div class="flex items-center gap-4">
			<a href="/docs" class="text-sm text-muted-foreground hover:text-foreground">Docs</a>
			<a href="/about" class="text-sm text-muted-foreground hover:text-foreground">About</a>
			<a href="/login">
				<Button variant="ghost">Sign In</Button>
			</a>
			<a href="/register">
				<Button>Get Started</Button>
			</a>
		</div>
	</header>

	<!-- Hero -->
	<main class="flex flex-1 flex-col items-center justify-center px-6 text-center">
		<div class="max-w-3xl space-y-8">
			<div class="space-y-4">
				<h1 class="text-4xl font-bold tracking-tight sm:text-6xl">
					Deploy containers with
					<span class="bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
						ease
					</span>
				</h1>
				<p class="mx-auto max-w-2xl text-lg text-muted-foreground">
					Labuh is a lightweight PaaS platform for deploying, pulling, and building container images.
					Simple, fast, and resource-efficient for your small servers.
				</p>
			</div>

			<div class="flex flex-wrap items-center justify-center gap-4">
				<a href="/register">
					<Button size="lg" class="gap-2">
						Start Deploying
						<ArrowRight class="h-4 w-4" />
					</Button>
				</a>
				<a href="https://github.com/HasanH47/labuh" target="_blank">
					<Button variant="outline" size="lg">
						View on GitHub
					</Button>
				</a>
			</div>
		</div>
	</main>

	<!-- Features -->
	<section class="border-t bg-muted/30 px-6 py-16">
		<div class="mx-auto max-w-5xl">
			<h2 class="mb-12 text-center text-2xl font-bold">Why Labuh?</h2>
			<div class="grid gap-8 md:grid-cols-3">
				<div class="space-y-3 text-center">
					<div class="mx-auto flex h-12 w-12 items-center justify-center rounded-lg bg-blue-500/10 text-blue-500">
						<Zap class="h-6 w-6" />
					</div>
					<h3 class="font-semibold">Lightweight</h3>
					<p class="text-sm text-muted-foreground">
						Built with Rust for maximum performance and minimal resource usage.
						Perfect for small VPS and home servers.
					</p>
				</div>
				<div class="space-y-3 text-center">
					<div class="mx-auto flex h-12 w-12 items-center justify-center rounded-lg bg-purple-500/10 text-purple-500">
						<Container class="h-6 w-6" />
					</div>
					<h3 class="font-semibold">Container Native</h3>
					<p class="text-sm text-muted-foreground">
						Direct containerd integration without the overhead of Docker daemon.
						Deploy any OCI-compatible container.
					</p>
				</div>
				<div class="space-y-3 text-center">
					<div class="mx-auto flex h-12 w-12 items-center justify-center rounded-lg bg-green-500/10 text-green-500">
						<Shield class="h-6 w-6" />
					</div>
					<h3 class="font-semibold">Auto SSL</h3>
					<p class="text-sm text-muted-foreground">
						Built-in Caddy reverse proxy with automatic Let's Encrypt SSL certificates.
						Secure by default.
					</p>
				</div>
			</div>
		</div>
	</section>

	<!-- Footer -->
	<footer class="border-t px-6 py-8 text-center text-sm text-muted-foreground">
		<p>© 2026 Labuh. Built with ❤️ using Rust and SvelteKit.</p>
	</footer>
</div>
