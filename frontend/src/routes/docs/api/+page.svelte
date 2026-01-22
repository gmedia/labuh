<script lang="ts">
	import * as Card from '$lib/components/ui/card';
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold tracking-tight">API Reference</h1>
		<p class="mt-2 text-muted-foreground">Complete REST API documentation</p>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title>Base URL</Card.Title>
		</Card.Header>
		<Card.Content>
			<code class="bg-muted px-2 py-1 rounded">http://localhost:3000/api</code>
			<p class="mt-2 text-sm text-muted-foreground">All endpoints require the <code>/api</code> prefix.</p>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Authentication</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<p>Most endpoints require authentication via Bearer token in the Authorization header:</p>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>Authorization: Bearer &lt;your-jwt-token&gt;</code></pre>

			<h3>POST /auth/register</h3>
			<p>Register a new user.</p>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>{`{
  "email": "user@example.com",
  "password": "password123",
  "name": "John Doe"
}`}</code></pre>

			<h3>POST /auth/login</h3>
			<p>Login and get JWT token.</p>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>{`{
  "email": "user@example.com",
  "password": "password123"
}`}</code></pre>
			<p><strong>Response:</strong></p>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>{`{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": { "id": "...", "email": "...", "name": "..." }
}`}</code></pre>

			<h3>GET /auth/me</h3>
			<p>Get current user info (requires auth).</p>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Containers</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>GET /containers</h3>
			<p>List all containers. Use <code>?all=true</code> to include stopped containers.</p>

			<h3>POST /containers</h3>
			<p>Create a new container.</p>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>{`{
  "name": "my-container",
  "image": "nginx:latest",
  "env": ["KEY=value"]
}`}</code></pre>

			<h3>POST /containers/:id/start</h3>
			<p>Start a stopped container.</p>

			<h3>POST /containers/:id/stop</h3>
			<p>Stop a running container.</p>

			<h3>POST /containers/:id/restart</h3>
			<p>Restart a container.</p>

			<h3>DELETE /containers/:id</h3>
			<p>Remove a container.</p>

			<h3>GET /containers/:id/logs</h3>
			<p>Get container logs. Use <code>?tail=100</code> to limit lines.</p>

			<h3>GET /containers/:id/stats</h3>
			<p>Get container resource stats (CPU, memory, network).</p>

			<h3>GET /containers/:id/logs/stream</h3>
			<p>Stream container logs via SSE (Server-Sent Events).</p>

			<h3>GET /containers/:id/stats/stream</h3>
			<p>Stream container stats via SSE.</p>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Images</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>GET /images</h3>
			<p>List all local images.</p>

			<h3>POST /images/pull</h3>
			<p>Pull an image from registry.</p>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>{`{
  "image": "nginx:latest"
}`}</code></pre>

			<h3>DELETE /images/:id</h3>
			<p>Remove an image.</p>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Projects</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>GET /projects</h3>
			<p>List all projects for the current user.</p>

			<h3>POST /projects</h3>
			<p>Create a new project.</p>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>{`{
  "name": "my-app",
  "description": "My application",
  "image": "nginx:latest",
  "port": 80,
  "env_vars": { "KEY": "value" }
}`}</code></pre>

			<h3>GET /projects/:id</h3>
			<p>Get project details.</p>

			<h3>PUT /projects/:id</h3>
			<p>Update a project.</p>

			<h3>DELETE /projects/:id</h3>
			<p>Delete a project.</p>

			<h3>POST /projects/:id/deploy</h3>
			<p>Deploy a project (pull image, create container, start).</p>

			<h3>POST /projects/:id/stop</h3>
			<p>Stop a deployed project.</p>

			<h3>POST /projects/:id/restart</h3>
			<p>Restart a deployed project.</p>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>System</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>GET /health</h3>
			<p>Health check endpoint (no auth required).</p>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>{`{
  "status": "ok",
  "version": "0.1.0"
}`}</code></pre>

			<h3>GET /system/stats</h3>
			<p>Get system resource stats (no auth required).</p>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>{`{
  "cpu_count": 4,
  "memory_total_kb": 8000000,
  "memory_available_kb": 4000000,
  "memory_used_percent": 50.0,
  "disk_total_bytes": 100000000000,
  "disk_available_bytes": 50000000000,
  "disk_used_percent": 50.0,
  "uptime_seconds": 86400,
  "load_average": { "one": 0.5, "five": 0.4, "fifteen": 0.3 }
}`}</code></pre>
		</Card.Content>
	</Card.Root>
</div>
