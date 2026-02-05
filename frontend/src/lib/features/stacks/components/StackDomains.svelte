<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import { Badge } from '$lib/components/ui/badge';
  import { Globe, CheckCircle, AlertCircle, Trash2, Radio, ExternalLink } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');
</script>

<Card.Root>
  <Card.Header>
    <div class="flex items-center justify-between">
      <Card.Title class="flex items-center gap-2">
        <Globe class="h-5 w-5" />
        Domains
      </Card.Title>
    </div>
    <Card.Description>Attached domains and routing</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-6">
    <!-- Redirection to Centralized Dashboard -->
    <div class="p-4 border border-dashed rounded-lg bg-muted/30 text-center space-y-3">
        <div class="bg-primary/10 w-10 h-10 rounded-full flex items-center justify-center mx-auto">
            <Globe class="h-5 w-5 text-primary" />
        </div>
        <div class="space-y-1">
            <p class="text-sm font-medium text-foreground">Centralized Management</p>
            <p class="text-[11px] text-muted-foreground max-w-[200px] mx-auto leading-tight">
                Register and attach new domains from the central dashboard.
            </p>
        </div>
        <Button variant="outline" size="sm" class="gap-2 h-8 text-xs font-normal" href="/dashboard/domains">
            Manage Domains
        </Button>
    </div>

    <div class="space-y-2">
      <Label class="text-[11px] uppercase tracking-wider font-bold text-muted-foreground/70">Attached to this Stack</Label>
      {#if ctrl.domains.length === 0}
        <div class="text-xs text-muted-foreground text-center py-6 border border-dashed rounded-lg">
            No domains attached to this stack
        </div>
      {:else}
        <div class="space-y-2">
            {#each ctrl.domains as domain}
            <div class="flex items-center justify-between p-3 rounded-lg border bg-background/50 hover:bg-muted/30 transition-colors group">
                <div class="flex items-center gap-3 overflow-hidden">
                <div class="flex-shrink-0">
                    {#if domain.type === 'Tunnel'}
                        <div class="h-8 w-8 rounded-full bg-blue-500/10 flex items-center justify-center overflow-hidden">
                            {#if domain.provider === 'Cloudflare'}
                                <img src="/logo/cloudflare/logo-1.png" alt="CF" class="h-5 w-5 object-contain" />
                            {:else}
                                <Radio class="h-4 w-4 text-blue-500" />
                            {/if}
                        </div>
                    {:else}
                        <div class="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center overflow-hidden">
                            {#if domain.provider === 'Cloudflare'}
                                <img src="/logo/cloudflare/logo-1.png" alt="CF" class="h-5 w-5 object-contain" />
                            {:else}
                                <Globe class="h-4 w-4 text-primary" />
                            {/if}
                        </div>
                    {/if}
                </div>
                <div class="flex flex-col overflow-hidden">
                    <div class="flex items-center gap-2">
                        <a href="https://{domain.domain}" target="_blank" class="text-sm font-semibold truncate hover:underline flex items-center gap-1">
                            {domain.domain}
                            <ExternalLink class="h-3 w-3 opacity-0 group-hover:opacity-50 transition-opacity" />
                        </a>
                        {#if domain.verified}
                            <CheckCircle class="h-3 w-3 text-green-500" />
                        {:else}
                            <AlertCircle class="h-3 w-3 text-yellow-500" />
                        {/if}
                    </div>
                    <span class="text-[11px] text-muted-foreground truncate">
                        {domain.container_name}:{domain.container_port} ({domain.provider})
                    </span>
                </div>
                </div>
                <div class="flex items-center gap-1">
                {#if domain.type === 'Caddy' && !isViewer}
                    <label class="flex items-center gap-1 cursor-pointer px-2 py-1 rounded hover:bg-muted/50 transition-colors" title="Toggle Labuh badge">
                        <input
                            type="checkbox"
                            checked={domain.show_branding}
                            onchange={() => ctrl.toggleBranding(domain.domain, !domain.show_branding)}
                            class="w-3.5 h-3.5 rounded border-muted-foreground/50 text-primary focus:ring-primary/50"
                        />
                        <span class="text-[10px] text-muted-foreground font-medium">Badge</span>
                    </label>
                {/if}
                {#if !isViewer}
                    <Button variant="ghost" size="icon" class="h-8 w-8 text-destructive hover:bg-destructive/10" onclick={() => ctrl.requestRemoveDomain(domain.domain)} title="Remove attachment">
                    <Trash2 class="h-4 w-4" />
                    </Button>
                {/if}
                </div>
            </div>
            {/each}
        </div>
      {/if}
    </div>
  </Card.Content>
</Card.Root>
