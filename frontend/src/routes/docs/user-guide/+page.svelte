<script lang="ts">
	import * as Card from '$lib/components/ui/card';
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold tracking-tight">User Guide</h1>
		<p class="mt-2 text-muted-foreground">Learn how to use all features of Labuh</p>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title>Getting Started</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>1. Create an Account</h3>
			<ol>
				<li>Open the Labuh dashboard in your browser</li>
				<li>Click <strong>Register</strong> on the landing page</li>
				<li>Enter your email and password</li>
				<li>Click <strong>Create Account</strong></li>
			</ol>

			<h3>2. Login</h3>
			<p>Use your registered email and password to login. You'll be redirected to the dashboard.</p>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Managing Containers</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>View Containers</h3>
			<p>Go to <strong>Dashboard → Containers</strong> to see all containers. Click on a container name to view details, logs, and stats.</p>

			<h3>Create a Container</h3>
			<ol>
				<li>Click <strong>Create Container</strong> button</li>
				<li>Enter a name (e.g., <code>my-nginx</code>)</li>
				<li>Enter image (e.g., <code>nginx:latest</code>)</li>
				<li>Click <strong>Create</strong></li>
			</ol>

			<h3>Container Actions</h3>
			<table>
				<thead>
					<tr><th>Button</th><th>Action</th></tr>
				</thead>
				<tbody>
					<tr><td>▶️ Play</td><td>Start stopped container</td></tr>
					<tr><td>⏹️ Stop</td><td>Stop running container</td></tr>
					<tr><td>🔄 Restart</td><td>Restart container</td></tr>
					<tr><td>🗑️ Delete</td><td>Remove container</td></tr>
				</tbody>
			</table>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Managing Images</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>Pull an Image</h3>
			<ol>
				<li>Go to <strong>Dashboard → Images</strong></li>
				<li>Enter image name (e.g., <code>nginx:latest</code>)</li>
				<li>Click <strong>Pull</strong></li>
			</ol>

			<h3>Image Sources</h3>
			<ul>
				<li><code>nginx:latest</code> - Docker Hub</li>
				<li><code>ghcr.io/user/repo:tag</code> - GitHub Container Registry</li>
				<li><code>your-registry.com/image:tag</code> - Private registry</li>
			</ul>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Managing Projects</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>Create a Project</h3>
			<ol>
				<li>Go to <strong>Dashboard → Projects</strong></li>
				<li>Click <strong>Create Project</strong></li>
				<li>Fill in:
					<ul>
						<li><strong>Name</strong> - Project name</li>
						<li><strong>Description</strong> - Optional</li>
						<li><strong>Docker Image</strong> - Image to deploy</li>
						<li><strong>Container Port</strong> - Port the app listens on</li>
					</ul>
				</li>
				<li>Click <strong>Create Project</strong></li>
			</ol>

			<h3>Deploy a Project</h3>
			<p>Click the <strong>🚀 Deploy</strong> button on a project card. Labuh will:</p>
			<ol>
				<li>Pull the latest image</li>
				<li>Create a container</li>
				<li>Start the container</li>
				<li>Configure routing</li>
			</ol>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Viewing Logs</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>Centralized Logs</h3>
			<p>Go to <strong>Dashboard → Logs</strong> to view logs from all containers in one place.</p>
			<ul>
				<li>Select a container from the sidebar</li>
				<li>Use the search box to filter logs</li>
				<li>Click <strong>Download</strong> to save logs as a file</li>
			</ul>

			<h3>Container Detail Logs</h3>
			<p>Click on a container name to view its detail page, which includes real-time logs and resource stats.</p>
		</Card.Content>
	</Card.Root>
</div>
