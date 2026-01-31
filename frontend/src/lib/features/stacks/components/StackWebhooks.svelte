<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Webhook, Copy } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';
  import { activeTeam } from '$lib/stores';
  import type { StackController } from '../stack-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: StackController }>();
  const isViewer = $derived($activeTeam?.role === 'Viewer');

  let selectedWebhookService = $state('');

  const filteredWebhookUrl = $derived(
    selectedWebhookService
      ? `${ctrl.webhookUrl}?service=${selectedWebhookService}`
      : ctrl.webhookUrl
  );

  function copyToClipboard(text: string) {
    if (navigator.clipboard) {
      navigator.clipboard.writeText(text).then(() => {
        toast.success('Copied to clipboard');
      }).catch(() => {
        fallbackCopyTextToClipboard(text);
      });
    } else {
      fallbackCopyTextToClipboard(text);
    }
  }

  function fallbackCopyTextToClipboard(text: string) {
    const textArea = document.createElement("textarea");
    textArea.value = text;
    textArea.style.top = "0";
    textArea.style.left = "0";
    textArea.style.position = "fixed";
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    try {
      const successful = document.execCommand('copy');
      if (successful) {
        toast.success('Copied to clipboard');
      } else {
        toast.error('Failed to copy');
      }
    } catch (err) {
      toast.error('Failed to copy');
    }
    document.body.removeChild(textArea);
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Webhook class="h-5 w-5" />
      Webhook
    </Card.Title>
    <Card.Description>Trigger deployments automatically</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    {#if ctrl.stack?.webhook_token}
      <div class="space-y-4">
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <Label>Webhook URL</Label>
            <select
              bind:value={selectedWebhookService}
              class="h-7 rounded-md border border-input bg-background px-2 text-[10px]"
            >
              <option value="">All Services</option>
              {#each ctrl.containers as container}
                <option value={container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}>
                  Only {container.labels?.['labuh.service.name'] || container.names[0]?.replace(/^\//, '')}
                </option>
              {/each}
            </select>
          </div>
          <div class="flex items-center gap-2">
            <Input readonly value={filteredWebhookUrl} class="font-mono text-[10px]" />
            <Button variant="outline" size="icon" class="h-9 w-9" onclick={() => copyToClipboard(filteredWebhookUrl)}>
              <Copy class="h-4 w-4" />
            </Button>
          </div>
          <p class="text-[10px] text-muted-foreground">
            {#if selectedWebhookService}
              POST to this URL to pull latest image and redeploy <strong>{selectedWebhookService}</strong> only.
            {:else}
              POST to this URL to pull latest images and redeploy all containers in this stack.
            {/if}
          </p>
        </div>
        <Button variant="outline" size="sm" class="w-full text-xs" onclick={() => ctrl.regenerateWebhook()} disabled={isViewer}>
          Regenerate Token
        </Button>
      </div>
    {:else}
      <Button class="w-full" onclick={() => ctrl.regenerateWebhook()}>
        Generate Webhook
      </Button>
    {/if}
  </Card.Content>
</Card.Root>
