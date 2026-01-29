<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { FileText, Server, Code, ArrowRight } from '@lucide/svelte';
</script>

<div class="space-y-8">
	<div>
		<h1 class="text-4xl font-bold tracking-tight">Labuh Documentation</h1>
		<p class="mt-2 text-lg text-muted-foreground">
			Everything you need to know about Labuh - the lightweight PaaS platform
		</p>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title>What is Labuh?</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<p>
				<strong>Labuh</strong> (dari bahasa Indonesia: berlabuh = to dock) adalah platform PaaS
				sederhana dan ringan untuk mengelola Docker containers, image, dan stacks. Dirancang khusus
				untuk memberikan kemudahan manajemen infrastruktur pada server dengan resource terbatas.
			</p>
			<h3>Features</h3>
			<ul>
				<li><strong>Container Management</strong> - Create, start, stop, restart, and remove containers</li>
				<li><strong>Image Management</strong> - Pull, list, and delete container images</li>
				<li><strong>Project Deployment</strong> - Deploy applications with one click</li>
				<li><strong>Real-time Monitoring</strong> - View logs, stats, and system metrics</li>
				<li><strong>Dark Mode</strong> - Beautiful UI with light and dark themes</li>
			</ul>
		</Card.Content>
	</Card.Root>

	<div class="grid gap-4 md:grid-cols-3">
		<a href="/docs/user-guide" class="group">
			<Card.Root class="transition-colors hover:border-primary">
				<Card.Header>
					<FileText class="h-8 w-8 text-primary" />
					<Card.Title class="group-hover:text-primary">User Guide</Card.Title>
				</Card.Header>
				<Card.Content>
					<p class="text-sm text-muted-foreground">
						Learn how to use all features of Labuh, from containers to projects.
					</p>
				</Card.Content>
				<Card.Footer>
					<span class="flex items-center text-sm text-primary">
						Read more <ArrowRight class="ml-1 h-4 w-4" />
					</span>
				</Card.Footer>
			</Card.Root>
		</a>

		<a href="/docs/deployment" class="group">
			<Card.Root class="transition-colors hover:border-primary">
				<Card.Header>
					<Server class="h-8 w-8 text-primary" />
					<Card.Title class="group-hover:text-primary">Deployment</Card.Title>
				</Card.Header>
				<Card.Content>
					<p class="text-sm text-muted-foreground">
						Install Labuh on your server using Docker, systemd, or manual setup.
					</p>
				</Card.Content>
				<Card.Footer>
					<span class="flex items-center text-sm text-primary">
						Read more <ArrowRight class="ml-1 h-4 w-4" />
					</span>
				</Card.Footer>
			</Card.Root>
		</a>

		<a href="/docs/api" class="group">
			<Card.Root class="transition-colors hover:border-primary">
				<Card.Header>
					<Code class="h-8 w-8 text-primary" />
					<Card.Title class="group-hover:text-primary">API Reference</Card.Title>
				</Card.Header>
				<Card.Content>
					<p class="text-sm text-muted-foreground">
						Complete API documentation with examples for all endpoints.
					</p>
				</Card.Content>
				<Card.Footer>
					<span class="flex items-center text-sm text-primary">
						Read more <ArrowRight class="ml-1 h-4 w-4" />
					</span>
				</Card.Footer>
			</Card.Root>
		</a>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title>Quick Start</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code># Clone repository
git clone https://github.com/HasanH47/labuh.git
cd labuh

# Start with Docker Compose
docker-compose up -d

# Open dashboard
open http://localhost:5173</code></pre>
		</Card.Content>
	</Card.Root>
</div>
