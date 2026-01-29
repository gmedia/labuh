<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Ship, Users, Zap, Heart, Target, Code, ArrowRight } from '@lucide/svelte';
</script>

<svelte:head>
	<title>About - Labuh</title>
</svelte:head>

<div class="min-h-screen bg-background">
	<!-- Header -->
	<header class="sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur">
		<div class="container mx-auto flex h-14 items-center justify-between px-4">
			<a href="/" class="flex items-center gap-2 font-semibold">
				<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary text-primary-foreground">
					<Ship class="h-6 w-6" />
				</div>
				<span>Labuh</span>
			</a>
			<div class="flex items-center gap-4">
				<a href="/docs" class="text-sm text-muted-foreground hover:text-foreground">Docs</a>
				<a href="/login">
					<Button variant="ghost">Sign In</Button>
				</a>
			</div>
		</div>
	</header>

	<main class="container mx-auto px-4 py-12">
		<!-- Hero Section -->
		<div class="max-w-3xl mx-auto text-center mb-16">
			<div class="flex justify-center mb-6">
				<div class="flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 to-purple-600 text-white">
					<Ship class="h-10 w-10" />
				</div>
			</div>
			<h1 class="text-4xl font-bold tracking-tight mb-4">
				Tentang Labuh
			</h1>
			<p class="text-xl text-muted-foreground">
				Platform PaaS sederhana untuk semua orang
			</p>
		</div>

		<!-- Why Section -->
		<Card.Root class="max-w-4xl mx-auto mb-8">
			<Card.Header>
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-500/10 text-blue-500">
						<Target class="h-5 w-5" />
					</div>
					<Card.Title>Mengapa Labuh?</Card.Title>
				</div>
			</Card.Header>
			<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
				<p>
					<strong>Labuh</strong> lahir dari kebutuhan sederhana: membuat container deployment
					menjadi lebih mudah dan accessible untuk semua orang.
				</p>
				<p>
					Banyak developer dan sysadmin yang sebenarnya ingin menggunakan container untuk
					deploy aplikasi mereka, tapi merasa overwhelmed dengan kompleksitas Command Line Interface (CLI)
					Docker, Compose, dan tool-tool lainnya. Mereka hanya butuh cara simple untuk:
				</p>
				<ul>
					<li>Pull image dari registry</li>
					<li>Jalankan container & Stacks</li>
					<li>Monitor logs & Health</li>
					<li>Setup domain & SSL secara otomatis</li>
				</ul>
				<p>
					Labuh hadir untuk menjembatani gap ini. Tanpa perlu hafal puluhan command,
					cukup dengan UI yang intuitif untuk mengimbangi engine Docker yang powerful.
				</p>
			</Card.Content>
		</Card.Root>

		<!-- For Who Section -->
		<Card.Root class="max-w-4xl mx-auto mb-8">
			<Card.Header>
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-purple-500/10 text-purple-500">
						<Users class="h-5 w-5" />
					</div>
					<Card.Title>Untuk Siapa?</Card.Title>
				</div>
			</Card.Header>
			<Card.Content>
				<div class="grid gap-4 md:grid-cols-2">
					<div class="p-4 rounded-lg border">
						<h4 class="font-semibold mb-2">🌱 Pemula</h4>
						<p class="text-sm text-muted-foreground">
							Baru belajar container? Labuh menyediakan interface visual yang mudah dipahami
							tanpa perlu menghafal command line.
						</p>
					</div>
					<div class="p-4 rounded-lg border">
						<h4 class="font-semibold mb-2">👨‍💻 Developer</h4>
						<p class="text-sm text-muted-foreground">
							Fokus ke coding, bukan ke infrastructure. Deploy aplikasi dengan satu klik,
							monitor logs secara real-time.
						</p>
					</div>
					<div class="p-4 rounded-lg border">
						<h4 class="font-semibold mb-2">🏠 Home Lab Enthusiast</h4>
						<p class="text-sm text-muted-foreground">
							Punya Raspberry Pi atau mini PC? Labuh didesain ringan untuk server dengan
							resource terbatas.
						</p>
					</div>
					<div class="p-4 rounded-lg border">
						<h4 class="font-semibold mb-2">🏢 Small Teams</h4>
						<p class="text-sm text-muted-foreground">
							Tim kecil yang butuh solusi deployment sederhana tanpa overhead Kubernetes
							atau cloud platform mahal.
						</p>
					</div>
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Features Section -->
		<Card.Root class="max-w-4xl mx-auto mb-8">
			<Card.Header>
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-green-500/10 text-green-500">
						<Zap class="h-5 w-5" />
					</div>
					<Card.Title>Keunggulan Labuh</Card.Title>
				</div>
			</Card.Header>
			<Card.Content>
				<div class="grid gap-4 md:grid-cols-3">
					<div class="text-center p-4">
						<div class="text-3xl font-bold text-primary mb-2">~ 50MB</div>
						<p class="text-sm text-muted-foreground">Binary size yang ringan</p>
					</div>
					<div class="text-center p-4">
						<div class="text-3xl font-bold text-primary mb-2">&lt; 100MB</div>
						<p class="text-sm text-muted-foreground">Memory usage saat idle</p>
					</div>
					<div class="text-center p-4">
						<div class="text-3xl font-bold text-primary mb-2">0 Config</div>
						<p class="text-sm text-muted-foreground">SQLite, tanpa database server</p>
					</div>
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Tech Section -->
		<Card.Root class="max-w-4xl mx-auto mb-8">
			<Card.Header>
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-orange-500/10 text-orange-500">
						<Code class="h-5 w-5" />
					</div>
					<Card.Title>Dibangun dengan Teknologi Modern</Card.Title>
				</div>
			</Card.Header>
			<Card.Content>
				<div class="grid gap-4 md:grid-cols-2">
					<div>
						<h4 class="font-semibold mb-2">Backend</h4>
						<ul class="text-sm text-muted-foreground space-y-1">
							<li>🦀 <strong>Rust</strong> - Performa tinggi, memory safe</li>
							<li>⚡ <strong>Axum</strong> - Async web framework</li>
							<li>🗄️ <strong>SQLite</strong> - Zero-config database</li>
							<li>🐳 <strong>Bollard</strong> - Docker API client</li>
						</ul>
					</div>
					<div>
						<h4 class="font-semibold mb-2">Frontend</h4>
						<ul class="text-sm text-muted-foreground space-y-1">
							<li>🔥 <strong>SvelteKit</strong> - Fast, reactive framework</li>
							<li>🎨 <strong>shadcn-svelte</strong> - Beautiful components</li>
							<li>💨 <strong>TailwindCSS</strong> - Utility-first CSS</li>
							<li>🌙 <strong>Dark Mode</strong> - Eye-friendly UI</li>
						</ul>
					</div>
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Open Source Section -->
		<Card.Root class="max-w-4xl mx-auto mb-8">
			<Card.Header>
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-red-500/10 text-red-500">
						<Heart class="h-5 w-5" />
					</div>
					<Card.Title>Open Source & Free</Card.Title>
				</div>
			</Card.Header>
			<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
				<p>
					Labuh adalah proyek open source. Kamu bisa menggunakannya secara gratis,
					memodifikasinya sesuai kebutuhan, dan berkontribusi untuk membuatnya lebih baik.
				</p>
				<p>
					Kami percaya bahwa tools untuk deployment seharusnya accessible untuk semua orang,
					bukan hanya untuk perusahaan besar dengan budget IT yang besar.
				</p>
			</Card.Content>
		</Card.Root>

		<!-- CTA -->
		<div class="text-center py-8">
			<h2 class="text-2xl font-bold mb-4">Siap Mencoba?</h2>
			<p class="text-muted-foreground mb-6">
				Mulai deploy aplikasi container kamu dengan Labuh sekarang.
			</p>
			<div class="flex justify-center gap-4">
				<a href="/register">
					<Button size="lg" class="gap-2">
						Mulai Sekarang
						<ArrowRight class="h-4 w-4" />
					</Button>
				</a>
				<a href="/docs">
					<Button variant="outline" size="lg">
						Baca Dokumentasi
					</Button>
				</a>
			</div>
		</div>
	</main>

	<!-- Footer -->
	<footer class="border-t px-6 py-8 text-center text-sm text-muted-foreground">
		<p>© 2026 Labuh. Built with ❤️ using Rust and SvelteKit.</p>
		<p class="mt-2">
			<a href="/" class="hover:text-foreground">Home</a>
			<span class="mx-2">•</span>
			<a href="/docs" class="hover:text-foreground">Docs</a>
			<span class="mx-2">•</span>
			<a href="/about" class="hover:text-foreground">About</a>
		</p>
	</footer>
</div>
