<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
	import { X, Globe, Save, Loader2 } from '@lucide/svelte';
	import type { DomainController } from '../domain-controller.svelte';

	let { ctrl } = $props<{ ctrl: DomainController }>();

	function close() {
		ctrl.showEditDnsDialog = false;
	}
</script>

{#if ctrl.showEditDnsDialog && ctrl.editingDomain}
<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto py-8">
    <Card.Root class="w-full max-w-lg mx-4">
        <Card.Header>
            <div class="flex items-center justify-between">
                <Card.Title class="flex items-center gap-2">
                    <Globe class="h-5 w-5 text-blue-500" />
                    Edit DNS: {ctrl.editingDomain.domain}
                </Card.Title>
                <Button variant="ghost" size="icon" onclick={close}>
                    <X class="h-4 w-4" />
                </Button>
            </div>
            <Card.Description>
                Update the DNS record for this domain at the provider.
            </Card.Description>
        </Card.Header>

        <Card.Content class="space-y-4">
            <div class="grid grid-cols-3 gap-4 p-4 border rounded-lg bg-muted/30">
                <div class="col-span-1 space-y-2">
                    <Label>Record Type</Label>
                    <Select.Root type="single" value={ctrl.editDnsType} onValueChange={(v) => v && (ctrl.editDnsType = v)}>
                        <Select.Trigger>
                            {ctrl.editDnsType}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="A">A</Select.Item>
                            <Select.Item value="CNAME">CNAME</Select.Item>
                            <Select.Item value="TXT">TXT</Select.Item>
                            <Select.Item value="MX">MX</Select.Item>
                        </Select.Content>
                    </Select.Root>
                </div>
                <div class="col-span-2 space-y-2">
                    <Label>Content / Value</Label>
                    <Input placeholder="1.2.3.4" bind:value={ctrl.editDnsContent} />
                </div>
            </div>

            <div class="p-3 border rounded-md bg-yellow-500/5 border-yellow-500/20 text-xs text-yellow-600 dark:text-yellow-400">
                <p><strong>Note:</strong> Changes will be sent directly to {ctrl.editingDomain.provider}. Propagation might take some time depending on TTL settings.</p>
            </div>
        </Card.Content>

        <Card.Footer class="flex justify-end gap-2 border-t pt-4">
            <Button variant="outline" onclick={close}>Cancel</Button>
            <Button disabled={ctrl.updatingDns} onclick={() => ctrl.updateDns()} class="gap-2">
                {#if ctrl.updatingDns}
                    <Loader2 class="h-4 w-4 animate-spin" />
                    Updating...
                {:else}
                    <Save class="h-4 w-4" />
                    Update DNS Record
                {/if}
            </Button>
        </Card.Footer>
    </Card.Root>
</div>
{/if}
