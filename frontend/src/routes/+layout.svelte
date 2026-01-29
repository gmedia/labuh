<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { theme } from '$lib/stores';

	// Apply theme on mount
	onMount(() => {
		const currentTheme = localStorage.getItem('theme') || 'light';
		document.documentElement.classList.toggle('dark', currentTheme === 'dark');
		theme.set(currentTheme as 'light' | 'dark');
	});

	import { Toaster } from 'svelte-sonner';

	let { children } = $props();
</script>

<svelte:head>
	<title>Labuh - Lightweight PaaS Platform</title>
	<meta name="description" content="Deploy, pull, and build container images with ease" />
	<link rel="preconnect" href="https://fonts.googleapis.com">
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
	<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
</svelte:head>

<div class="min-h-screen bg-background font-sans antialiased" style="font-family: 'Inter', sans-serif;">
	<Toaster position="top-right" richColors />
	{@render children()}
</div>
