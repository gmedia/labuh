<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Globe, Plus, Search, Loader2, Cloud, Download } from '@lucide/svelte';
	import { Input } from '$lib/components/ui/input';
	import type { DomainController } from '../domain-controller.svelte';
	import type { RemoteDnsRecord } from '$lib/api';

	let { ctrl } = $props<{ ctrl: DomainController }>();
	let searchQuery = $state('');

	const filteredRecords = $derived(
		ctrl.remoteRecords.filter((r: RemoteDnsRecord) =>
			r.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
			r.zone_name.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<div class="space-y-1">
			<h3 class="text-lg font-semibold flex items-center gap-2">
				<Search class="h-5 w-5 text-blue-500" />
				Discover Existing Records
			</h3>
			<p class="text-xs text-muted-foreground">
				Found records in your DNS providers that are not yet managed by Labuh.
			</p>
		</div>
		<div class="flex items-center gap-2">
			<div class="relative w-64">
				<Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
				<Input
					type="search"
					placeholder="Search records..."
					class="pl-8 h-9"
					bind:value={searchQuery}
				/>
			</div>
			{#if ctrl.dnsConfigs.length > 0}
				<Button
					size="sm"
					variant="outline"
					class="gap-2"
					onclick={() => ctrl.loadRemoteRecords(ctrl.dnsConfigs[0].provider)}
					disabled={ctrl.loadingRemote}
				>
					{#if ctrl.loadingRemote}
						<Loader2 class="h-4 w-4 animate-spin" />
					{:else}
						<Cloud class="h-4 w-4" />
					{/if}
					Refresh
				</Button>
			{/if}
		</div>
	</div>

	{#if ctrl.loadingRemote && ctrl.remoteRecords.length === 0}
		<div class="flex flex-col items-center justify-center py-12 text-center border rounded-lg bg-muted/20 border-dashed">
			<Loader2 class="h-8 w-8 animate-spin text-muted-foreground mb-4" />
			<p class="text-sm text-muted-foreground">Scanning DNS configurations...</p>
		</div>
	{:else if ctrl.remoteRecords.length === 0}
		<div class="flex flex-col items-center justify-center py-12 text-center border rounded-lg bg-muted/20 border-dashed">
			<Globe class="h-10 w-10 text-muted-foreground/30 mb-4" />
			<p class="text-sm text-muted-foreground">No unmanaged records found.</p>
			<p class="text-xs text-muted-foreground mt-1">Try refreshing if you just added records in your provider.</p>
		</div>
	{:else}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each filteredRecords as record}
				<Card.Root class="overflow-hidden hover:ring-1 hover:ring-primary/20 transition-all">
					<Card.Header class="p-4 pb-2">
						<div class="flex items-center justify-between mb-1">
							<Badge variant="outline" class="text-[10px] uppercase font-bold py-0 h-5">
								{record.type}
							</Badge>
							<span class="text-[10px] text-muted-foreground flex items-center gap-1">
								<Cloud class="h-3 w-3" />
								{record.zone_name}
							</span>
						</div>
						<Card.Title class="text-sm font-bold truncate" title={record.name}>
							{record.name}
						</Card.Title>
					</Card.Header>
					<Card.Content class="p-4 pt-0">
						<p class="text-[11px] text-muted-foreground truncate mb-4" title={record.content}>
							Points to: {record.content}
						</p>
						<Button
							variant="secondary"
							size="sm"
							class="w-full gap-2 h-8 text-xs"
							onclick={() => ctrl.openImportDialog(record)}
						>
							<Download class="h-3.5 w-3.5" />
							Import to Labuh
						</Button>
					</Card.Content>
				</Card.Root>
			{/each}
		</div>
	{/if}
</div>
