<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
	import { X, Globe, Plus, Loader2 } from '@lucide/svelte';
	import type { DomainController } from '../domain-controller.svelte';

	let { ctrl } = $props<{ ctrl: DomainController }>();

	function handleProviderChange(val: string) {
		if (val) {
			ctrl.selectedProvider = val;
			ctrl.fetchAvailableDomains(val);
		}
	}

	function handleStackChange(val: string) {
		if (val) {
			ctrl.selectedStackId = val;
			ctrl.fetchContainers(val);
		}
	}

    function handleContainerChange(val: string) {
        if (val) ctrl.selectedContainer = val;
    }
</script>

{#if ctrl.showRegisterDialog}
<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto py-8">
    <Card.Root class="w-full max-w-lg mx-4">
        <Card.Header>
            <div class="flex items-center justify-between">
                <Card.Title class="flex items-center gap-2">
                    <Globe class="h-5 w-5" />
                    Register & Attach Domain
                </Card.Title>
                <Button variant="ghost" size="icon" onclick={() => ctrl.showRegisterDialog = false}>
                    <X class="h-4 w-4" />
                </Button>
            </div>
            <Card.Description>
                Configure a new domain/subdomain and attach it to a running stack service.
            </Card.Description>
        </Card.Header>

        <Card.Content class="space-y-4">
            <div class="grid grid-cols-2 gap-4">
                <div class="space-y-2">
                    <Label>DNS Provider</Label>
                    <Select.Root type="single" value={ctrl.selectedProvider} onValueChange={handleProviderChange}>
                        <Select.Trigger class="w-full">
                            {ctrl.selectedProvider}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="Custom">Custom (Manual)</Select.Item>
                            {#each ctrl.dnsConfigs as config}
                                <Select.Item value={config.provider}>{config.provider}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>

                <div class="space-y-2">
                    <Label>Base Domain</Label>
                    {#if ctrl.selectedProvider === 'Custom' || ctrl.availableBaseDomains.length === 0}
                        <Input placeholder="example.com" bind:value={ctrl.selectedBaseDomain} />
                    {:else}
                        <Select.Root type="single" value={ctrl.selectedBaseDomain} onValueChange={(v) => v && (ctrl.selectedBaseDomain = v)}>
                            <Select.Trigger class="w-full">
                                {ctrl.selectedBaseDomain || "Select domain"}
                            </Select.Trigger>
                            <Select.Content>
                                {#each ctrl.availableBaseDomains as d}
                                    <Select.Item value={d}>{d}</Select.Item>
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    {/if}
                </div>
            </div>

            <div class="space-y-2">
                <Label for="subdomain">Subdomain (Optional)</Label>
                <div class="flex items-center gap-2">
                    <Input id="subdomain" placeholder="app" bind:value={ctrl.subdomain} class="text-right" />
                    <span class="text-muted-foreground">.{ctrl.selectedBaseDomain || "domain.com"}</span>
                </div>
            </div>

            <hr class="my-4 border-dashed" />

            <div class="grid grid-cols-2 gap-4">
                <div class="space-y-2">
                    <Label>Assign to Stack</Label>
                    <Select.Root type="single" value={ctrl.selectedStackId} onValueChange={handleStackChange}>
                        <Select.Trigger class="w-full">
                            {ctrl.stacks.find((s: any) => s.id === ctrl.selectedStackId)?.name || "Select Stack"}
                        </Select.Trigger>
                        <Select.Content>
                            {#each ctrl.stacks as stack}
                                <Select.Item value={stack.id}>{stack.name}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>

                <div class="space-y-2">
                    <Label>Service/Container</Label>
                    <Select.Root type="single" value={ctrl.selectedContainer} onValueChange={handleContainerChange}>
                        <Select.Trigger class="w-full">
                            {ctrl.selectedContainer || "Select Service"}
                        </Select.Trigger>
                        <Select.Content>
                            {#each ctrl.containers as c}
                                {#each c.names as n}
                                    <Select.Item value={n.replace('/', '')}>{n.replace('/', '')}</Select.Item>
                                {/each}
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <div class="space-y-2">
                <Label for="port">Container Port</Label>
                <Input id="port" type="number" bind:value={ctrl.selectedPort} />
                <p class="text-[10px] text-muted-foreground">The internal port your service is listening on (default: 80).</p>
            </div>
        </Card.Content>

        <Card.Footer class="flex justify-end gap-2">
            <Button variant="outline" onclick={() => ctrl.showRegisterDialog = false}>Cancel</Button>
            <Button disabled={ctrl.registrationLoading} onclick={() => ctrl.registerDomain()}>
                {#if ctrl.registrationLoading}
                    <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                    Registering...
                {:else}
                    Register Domain
                {/if}
            </Button>
        </Card.Footer>
    </Card.Root>
</div>
{/if}
