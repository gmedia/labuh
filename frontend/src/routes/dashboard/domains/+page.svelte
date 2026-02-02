<script lang="ts">
	import { onMount } from 'svelte';
	import { activeTeam } from '$lib/stores';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Globe, Plus, Settings2, Trash2, ShieldCheck, ShieldAlert, Cloud, Radio, ExternalLink } from '@lucide/svelte';
	import { DomainController } from '$lib/features/domains/domain-controller.svelte';
	import DnsConfigDialog from '$lib/features/domains/components/DnsConfigDialog.svelte';
	import RegisterDomainDialog from '$lib/features/domains/components/RegisterDomainDialog.svelte';
	import EditDnsDialog from '$lib/features/domains/components/EditDnsDialog.svelte';
	import DiscoveredRecords from '$lib/features/domains/components/DiscoveredRecords.svelte';
	import * as Tabs from '$lib/components/ui/tabs';
	import { formatDistanceToNow } from 'date-fns';

	let ctrl = $state(new DomainController());
    let currentTab = $state("managed");

	onMount(async () => {
		await ctrl.init();
	});

	$effect(() => {
		if ($activeTeam?.team) {
			ctrl.loadAll();
		}
	});
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold tracking-tight">Domains</h2>
			<p class="text-muted-foreground">Manage project domains and DNS automation</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" class="gap-2" onclick={() => ctrl.showDnsDialog = true}>
				<Settings2 class="h-4 w-4" />
				DNS Settings
			</Button>
			<Button class="gap-2 shadow-lg shadow-blue-500/20" onclick={() => ctrl.openRegisterDialog()}>
				<Plus class="h-4 w-4" />
				Add Domain
			</Button>
		</div>
	</div>

	<div class="grid gap-6 md:grid-cols-3">
		<!-- DNS Providers Configured -->
		<Card.Root class="md:col-span-1">
			<Card.Header>
				<Card.Title class="text-sm font-medium">DNS Provider</Card.Title>
			</Card.Header>
			<Card.Content>
				{#if ctrl.dnsConfigs.length === 0}
					<div class="flex flex-col items-center justify-center py-4 text-center">
						<Cloud class="mb-2 h-8 w-8 text-muted-foreground/30" />
						<p class="text-xs text-muted-foreground mb-2">No DNS providers configured</p>
						<Button size="sm" variant="outline" onclick={() => ctrl.showDnsDialog = true}>Configure</Button>
					</div>
				{:else}
					<div class="space-y-2">
						{#each ctrl.dnsConfigs as config}
							<div class="flex items-center justify-between border rounded-lg p-2 bg-muted/50 transition-all hover:bg-muted">
								<div class="flex items-center gap-2">
									<div class="bg-blue-500/10 p-1.5 rounded-md">
										<Cloud class="h-4 w-4 text-blue-500" />
									</div>
									<span class="text-sm font-medium">{config.provider}</span>
								</div>
								<Button
									size="icon"
									variant="ghost"
									class="h-8 w-8 text-destructive hover:bg-destructive/10"
									onclick={() => ctrl.removeDnsConfig(config.provider)}
								>
									<Trash2 class="h-4 w-4" />
								</Button>
							</div>
						{/each}
					</div>
				{/if}
			</Card.Content>
		</Card.Root>

		<!-- Domain Statistics -->
		<Card.Root class="md:col-span-2">
			<Card.Header>
				<Card.Title class="text-sm font-medium">Overview</Card.Title>
			</Card.Header>
			<Card.Content>
				<div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
					<div class="space-y-1">
						<p class="text-xs text-muted-foreground">Total Domains</p>
						<p class="text-2xl font-bold">{ctrl.domains.length}</p>
					</div>
					<div class="space-y-1">
						<p class="text-xs text-muted-foreground">Verified</p>
						<p class="text-2xl font-bold text-green-500">
							{ctrl.domains.filter((d) => d.verified).length}
						</p>
					</div>
					<div class="space-y-1">
						<p class="text-xs text-muted-foreground">Tunnels</p>
						<p class="text-2xl font-bold text-blue-500">
							{ctrl.domains.filter((d) => d.type === 'Tunnel').length}
						</p>
					</div>
					<div class="space-y-1">
						<p class="text-xs text-muted-foreground">SSL Enabled</p>
						<p class="text-2xl font-bold text-cyan-500">
							{ctrl.domains.filter((d) => d.ssl_enabled).length}
						</p>
					</div>
				</div>
			</Card.Content>
		</Card.Root>
	</div>

	<Tabs.Root bind:value={currentTab} class="space-y-6">
        <Tabs.List>
            <Tabs.Trigger value="managed">Managed Domains</Tabs.Trigger>
            <Tabs.Trigger value="discovered" onclick={() => ctrl.dnsConfigs.length > 0 && ctrl.loadRemoteRecords(ctrl.dnsConfigs[0].provider)}>
                Discover Records
            </Tabs.Trigger>
        </Tabs.List>

        <Tabs.Content value="managed">
            {#if ctrl.loading}
                <div class="flex items-center justify-center py-12">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                </div>
            {:else if ctrl.domains.length === 0}
                <Card.Root>
                    <Card.Content class="flex flex-col items-center justify-center py-16 text-center">
                        <div class="bg-muted p-4 rounded-full mb-4">
                            <Globe class="h-12 w-12 text-muted-foreground/50" />
                        </div>
                        <h3 class="text-lg font-semibold">No domains registered</h3>
                        <p class="mb-4 text-sm text-muted-foreground">
                            Register a domain and attach it to your services.
                        </p>
                        <Button class="gap-2" onclick={() => ctrl.openRegisterDialog()}>
                            <Plus class="h-4 w-4" />
                            Add Your First Domain
                        </Button>
                    </Card.Content>
                </Card.Root>
            {:else}
                <div class="rounded-md border bg-card text-card-foreground shadow-sm">
                    <div
                        class="grid grid-cols-6 items-center border-b bg-muted/30 p-4 text-sm font-medium text-muted-foreground"
                    >
                        <div class="col-span-2">Domain</div>
                        <div>Type</div>
                        <div>Provider</div>
                        <div>Status</div>
                        <div class="hidden md:block text-xs">Registered</div>
                        <div class="text-right pr-4">Actions</div>
                    </div>
                    <div class="divide-y">
                        {#each ctrl.domains as domain}
                            <div class="grid grid-cols-6 items-center p-4 text-sm hover:bg-muted/30 transition-colors">
                                <div class="col-span-2 flex flex-col">
                                    <a
                                        href="https://{domain.domain}"
                                        target="_blank"
                                        class="font-semibold text-blue-600 dark:text-blue-400 hover:underline flex items-center gap-1 group w-fit"
                                    >
                                        {domain.domain}
                                        <ExternalLink class="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity" />
                                    </a>
                                    <span class="text-xs text-muted-foreground mt-0.5">
                                        Linked to <span class="font-medium text-foreground">{domain.container_name}</span>
                                    </span>
                                </div>
                                <div>
                                    {#if domain.type === 'Tunnel'}
                                        <Badge variant="outline" class="gap-1 border-blue-500/30 text-blue-500 bg-blue-500/5">
                                            <Radio class="h-3 w-3" />
                                            Tunnel
                                        </Badge>
                                    {:else}
                                        <Badge variant="outline" class="gap-1 font-normal text-muted-foreground"> Caddy </Badge>
                                    {/if}
                                </div>
                                <div>
                                    <span class="text-xs flex items-center gap-1 text-muted-foreground">
                                        {#if domain.provider === 'Cloudflare'}
                                            <Cloud class="h-3 w-3 text-blue-400" />
                                            Cloudflare
                                        {:else if domain.provider === 'CPanel'}
                                            <Settings2 class="h-3 w-3 text-orange-400" />
                                            cPanel
                                        {:else}
                                            <Globe class="h-3 w-3" />
                                            Custom
                                        {/if}
                                    </span>
                                </div>
                                <div>
                                    {#if domain.verified}
                                        <Badge variant="outline" class="gap-1 text-green-500 border-green-500/30 bg-green-500/5 font-normal">
                                            <ShieldCheck class="h-3 w-3" />
                                            Verified
                                        </Badge>
                                    {:else}
                                        <button onclick={() => ctrl.verifyDomain(domain.stack_id, domain.domain)}>
                                            <Badge
                                                variant="outline"
                                                class="gap-1 text-yellow-500 border-yellow-500/30 bg-yellow-500/5 cursor-pointer hover:bg-yellow-500/10 transition-colors font-normal"
                                            >
                                                <ShieldAlert class="h-3 w-3" />
                                                Verify
                                            </Badge>
                                        </button>
                                    {/if}
                                </div>
                                <div class="hidden md:block text-xs text-muted-foreground">
                                    {domain.created_at ? formatDistanceToNow(new Date(domain.created_at), { addSuffix: true }) : '-'}
                                </div>
                                <div class="text-right pr-2 flex items-center justify-end gap-1">
                                    {#if domain.provider !== 'Custom'}
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            class="h-8 w-8 text-muted-foreground hover:text-foreground"
                                            onclick={() => ctrl.openEditDns(domain)}
                                            title="Edit DNS"
                                        >
                                            <Settings2 class="h-4 w-4" />
                                        </Button>
                                    {/if}
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        class="text-destructive h-8 w-8 hover:bg-destructive/10"
                                        onclick={() => ctrl.removeDomain(domain.stack_id, domain.domain)}
                                        title="Delete Domain"
                                    >
                                        <Trash2 class="h-4 w-4" />
                                    </Button>
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
        </Tabs.Content>

        <Tabs.Content value="discovered">
            <DiscoveredRecords {ctrl} />
        </Tabs.Content>
    </Tabs.Root>
</div>

<DnsConfigDialog {ctrl} />
<RegisterDomainDialog {ctrl} />
<EditDnsDialog {ctrl} />
