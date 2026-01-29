<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { auth, theme } from '$lib/stores';
	import { api } from '$lib/api';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { Button } from '$lib/components/ui/button';
	import {
		Home,
		Container,
		Image,
		FolderKanban,
		Layers,
		Settings,
		LogOut,
		Sun,
		Moon,
		Ship,
		Terminal
	} from '@lucide/svelte';

	let { children } = $props();

	const navItems = [
		{ href: '/dashboard', label: 'Dashboard', icon: Home },
		{ href: '/dashboard/containers', label: 'Containers', icon: Container },
		{ href: '/dashboard/images', label: 'Images', icon: Image },
		{ href: '/dashboard/stacks', label: 'Stacks', icon: Layers },
		{ href: '/dashboard/logs', label: 'Logs', icon: Terminal },
		{ href: '/dashboard/settings', label: 'Settings', icon: Settings },
	];

	function handleLogout() {
		api.auth.logout();
		goto('/login');
	}
</script>

<Sidebar.Provider>
	<Sidebar.Root>
		<Sidebar.Header class="p-4">
			<div class="flex items-center gap-2">
				<div class="flex h-10 w-10 items-center justify-center rounded-lg overflow-hidden">
					<img src="/logo.png" alt="Labuh Logo" class="h-8 w-8 object-contain" />
				</div>
				<div>
					<h1 class="text-lg font-bold">Labuh</h1>
					<p class="text-xs text-muted-foreground">PaaS Platform</p>
				</div>
			</div>
		</Sidebar.Header>

		<Sidebar.Content>
			<Sidebar.Group>
				<Sidebar.GroupLabel>Navigation</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each navItems as item}
							<Sidebar.MenuItem>
								<a
									href={item.href}
									class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground {$page.url.pathname === item.href ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium' : ''}"
								>
									<item.icon class="h-4 w-4" />
									<span>{item.label}</span>
								</a>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		</Sidebar.Content>

		<Sidebar.Footer class="p-4">
			<div class="flex items-center justify-between">
				<Button variant="ghost" size="icon" onclick={() => theme.toggle()}>
					{#if $theme === 'dark'}
						<Sun class="h-4 w-4" />
					{:else}
						<Moon class="h-4 w-4" />
					{/if}
				</Button>
				<Button variant="ghost" size="icon" onclick={handleLogout}>
					<LogOut class="h-4 w-4" />
				</Button>
			</div>
			{#if $auth.user}
				<div class="mt-2 text-sm">
					<p class="font-medium truncate">{$auth.user.email}</p>
					<p class="text-xs text-muted-foreground capitalize">{$auth.user.role}</p>
				</div>
			{/if}
		</Sidebar.Footer>
	</Sidebar.Root>

	<Sidebar.Inset>
		<header class="flex h-14 items-center gap-4 border-b px-6">
			<Sidebar.Trigger />
			<div class="flex-1"></div>
		</header>
		<main class="flex-1 p-6">
			{@render children()}
		</main>
	</Sidebar.Inset>
</Sidebar.Provider>
