<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
    import { X, Cloud } from '@lucide/svelte';
	import type { DnsConfig } from '$lib/api';
	import type { DomainController } from '../domain-controller.svelte';

	let { ctrl } = $props<{ ctrl: DomainController }>();

	const providers = [
		{ value: 'Cloudflare', label: 'Cloudflare' },
		{ value: 'CPanel', label: 'cPanel (Soon)', disabled: true }
	];

    $effect(() => {
        // Track provider change
        const currentProvider = ctrl.selectedProvider;

        // Load existing config if available when provider changes
        const existing = ctrl.dnsConfigs.find((c: DnsConfig) => c.provider === currentProvider);
        if (existing) {
            ctrl.dnsConfigFields = JSON.parse(existing.config);
        } else {
            ctrl.dnsConfigFields = {};
        }
    });

    function handleProviderChange(val: string) {
        if (val) ctrl.selectedProvider = val;
    }
</script>

{#if ctrl.showDnsDialog}
<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto py-8">
    <Card.Root class="w-full max-w-md mx-4">
        <Card.Header>
            <div class="flex items-center justify-between">
                <Card.Title class="flex items-center gap-2">
                    <Cloud class="h-5 w-5" />
                    DNS Provider Configuration
                </Card.Title>
                <Button variant="ghost" size="icon" onclick={() => ctrl.showDnsDialog = false}>
                    <X class="h-4 w-4" />
                </Button>
            </div>
            <Card.Description>
                Configure your DNS provider to enable automated record management.
            </Card.Description>
        </Card.Header>

        <Card.Content class="space-y-4">
            <div class="space-y-2">
                <Label>Provider</Label>
                <Select.Root type="single" value={ctrl.selectedProvider} onValueChange={handleProviderChange}>
                    <Select.Trigger class="w-full">
                        {ctrl.selectedProvider}
                    </Select.Trigger>
                    <Select.Content>
                        {#each providers as p}
                            <Select.Item value={p.value} disabled={p.disabled}>{p.label}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>

            {#if ctrl.selectedProvider === 'Cloudflare'}
                <div class="space-y-4 pt-2">
                    <div class="space-y-2">
                        <Label for="api_token">API Token</Label>
                        <Input
                            id="api_token"
                            type="password"
                            placeholder="Cloudflare API Token"
                            bind:value={ctrl.dnsConfigFields.api_token}
                        />
                        <p class="text-[10px] text-muted-foreground">
                            Requires 'Zone.DNS' edit permissions.
                        </p>
                    </div>
                    <div class="space-y-2">
                        <Label for="zone_id">Zone ID</Label>
                        <Input
                            id="zone_id"
                            placeholder="Cloudflare Zone ID"
                            bind:value={ctrl.dnsConfigFields.zone_id}
                        />
                    </div>
                </div>
            {/if}

            <div class="text-xs text-muted-foreground bg-muted p-3 rounded-md border">
                <p><strong>Note:</strong> Labuh will use these credentials to create A or CNAME records when you add a domain to a stack.</p>
            </div>
        </Card.Content>

        <Card.Footer class="flex justify-end gap-2">
            <Button variant="outline" onclick={() => ctrl.showDnsDialog = false}>Cancel</Button>
            <Button onclick={() => ctrl.saveDnsConfig()}>Save Configuration</Button>
        </Card.Footer>
    </Card.Root>
</div>
{/if}
