<script lang="ts">
	import { page } from '$app/stores';
	import * as Card from '$lib/components/ui/card';
	import { Book, FileText, Server, Code } from '@lucide/svelte';

	const navItems = [
		{ href: '/docs', label: 'Introduction', icon: Book },
		{ href: '/docs/user-guide', label: 'User Guide', icon: FileText },
		{ href: '/docs/deployment', label: 'Deployment', icon: Server },
		{ href: '/docs/api', label: 'API Reference', icon: Code },
	];
</script>

<div class="min-h-screen bg-background">
	<!-- Header -->
	<header class="sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur">
		<div class="container mx-auto flex h-14 items-center justify-between px-4">
			<div class="flex items-center gap-4">
				<a href="/" class="flex items-center gap-2 font-semibold">
					<span class="text-xl">⚓</span>
					<span>Labuh</span>
				</a>
				<span class="text-muted-foreground">/</span>
				<span class="text-muted-foreground">Documentation</span>
			</div>
			<div class="flex items-center gap-4">
				<a href="/dashboard" class="text-sm text-muted-foreground hover:text-foreground">Dashboard</a>
				<a href="https://github.com/HasanH47/labuh" target="_blank" class="text-sm text-muted-foreground hover:text-foreground">GitHub</a>
			</div>
		</div>
	</header>

	<div class="container mx-auto flex gap-8 px-4 py-8">
		<!-- Sidebar -->
		<aside class="hidden lg:block w-64 shrink-0">
			<nav class="sticky top-20 space-y-1">
				{#each navItems as item}
					<a
						href={item.href}
						class="flex items-center gap-2 rounded-lg px-3 py-2 transition-colors {$page.url.pathname === item.href ? 'bg-muted font-medium' : 'text-muted-foreground hover:bg-muted hover:text-foreground'}"
					>
						<item.icon class="h-4 w-4" />
						{item.label}
					</a>
				{/each}
			</nav>
		</aside>

		<!-- Main Content -->
		<main class="flex-1 min-w-0">
			<slot />
		</main>
	</div>
</div>
