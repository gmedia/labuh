<script lang="ts">
	import * as Card from '$lib/components/ui/card';
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold tracking-tight">Deployment Guide</h1>
		<p class="mt-2 text-muted-foreground">Install Labuh on your server</p>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title>Prerequisites</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<ul>
				<li><strong>Linux server</strong> (Ubuntu 20.04+ recommended)</li>
				<li><strong>Docker</strong> or <strong>containerd</strong> with nerdctl</li>
				<li><strong>1GB RAM</strong> minimum (2GB recommended)</li>
				<li>Ports 80, 443, 3000 available</li>
			</ul>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Option 1: Docker Compose (Recommended)</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code># Clone repository
git clone https://github.com/HasanH47/labuh.git
cd labuh

# Configure environment
cp .env.example .env
nano .env  # Edit JWT_SECRET

# Start services
docker-compose up -d

# Check status
docker-compose ps</code></pre>

			<h3>Services Started</h3>
			<table>
				<thead><tr><th>Service</th><th>Port</th><th>Description</th></tr></thead>
				<tbody>
					<tr><td>labuh</td><td>3000</td><td>Backend API</td></tr>
					<tr><td>caddy</td><td>80, 443</td><td>Reverse proxy</td></tr>
					<tr><td>frontend</td><td>5173</td><td>Web UI</td></tr>
				</tbody>
			</table>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Option 2: Systemd Service</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>Build from Source</h3>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code># Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/HasanH47/labuh.git
cd labuh
cargo build --release

# Run install script
sudo ./deploy/install.sh</code></pre>

			<h3>Manage Service</h3>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code># Start
sudo systemctl start labuh

# Stop
sudo systemctl stop labuh

# View logs
sudo journalctl -u labuh -f</code></pre>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Option 3: Containerd Only (No Docker)</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<p>For servers where you don't want the full Docker daemon:</p>

			<h3>Install containerd + nerdctl</h3>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code># Install containerd
sudo apt install containerd

# Install nerdctl (Docker-compatible CLI)
wget https://github.com/containerd/nerdctl/releases/download/v1.7.0/nerdctl-1.7.0-linux-amd64.tar.gz
sudo tar -xzf nerdctl-*.tar.gz -C /usr/local/bin

# Enable Docker API shim
# This allows bollard/Docker API clients to work with containerd
sudo nerdctl run -d --name dockerd-shim \
  --privileged \
  -v /run/containerd:/run/containerd \
  docker:dind</code></pre>

			<p><strong>Note:</strong> Labuh uses the Docker API via bollard. For containerd-only setups,
			you can use the lightweight Docker shim shown above, which provides Docker API compatibility
			without the full Docker daemon.</p>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Environment Variables</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<table>
				<thead><tr><th>Variable</th><th>Default</th><th>Description</th></tr></thead>
				<tbody>
					<tr><td><code>HOST</code></td><td>0.0.0.0</td><td>Listen address</td></tr>
					<tr><td><code>PORT</code></td><td>3000</td><td>Listen port</td></tr>
					<tr><td><code>DATABASE_URL</code></td><td>sqlite:./labuh.db</td><td>SQLite path</td></tr>
					<tr><td><code>JWT_SECRET</code></td><td>(required)</td><td>Secret for JWT tokens</td></tr>
					<tr><td><code>JWT_EXPIRATION_HOURS</code></td><td>24</td><td>Token lifetime</td></tr>
				</tbody>
			</table>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Backup & Restore</Card.Title>
		</Card.Header>
		<Card.Content class="prose prose-neutral dark:prose-invert max-w-none">
			<h3>Backup</h3>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>sudo ./deploy/backup.sh</code></pre>

			<h3>Restore</h3>
			<pre class="bg-muted p-4 rounded-lg overflow-x-auto"><code>sudo ./deploy/restore.sh /var/backups/labuh/labuh_backup_20260122.tar.gz</code></pre>
		</Card.Content>
	</Card.Root>
</div>
