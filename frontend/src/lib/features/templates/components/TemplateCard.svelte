<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { LayoutGrid, Globe, FileText, Database, Plus } from '@lucide/svelte';
  import { activeTeam } from '$lib/stores';
  import { goto } from '$app/navigation';
  import type { TemplateResponse } from '$lib/api';

  let { template } = $props<{ template: TemplateResponse }>();

  function getIcon(iconName: string) {
    switch (iconName) {
      case 'globe': return Globe;
      case 'file-text': return FileText;
      case 'database': return Database;
      default: return LayoutGrid;
    }
  }

  function deployTemplate(templateId: string) {
    goto(`/dashboard/stacks?template=${templateId}`);
  }

  const Icon = $derived(getIcon(template.icon));
</script>

<Card.Root class="flex flex-col h-full transition-all hover:shadow-md border-muted/60">
  <Card.Header>
    <div class="flex items-center gap-3">
      <div class="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
        <Icon class="h-6 w-6" />
      </div>
      <div>
        <Card.Title>{template.name}</Card.Title>
      </div>
    </div>
  </Card.Header>
  <Card.Content class="flex-1">
    <p class="text-sm text-muted-foreground line-clamp-3">
      {template.description}
    </p>
  </Card.Content>
  <Card.Footer class="border-t bg-muted/5 py-3">
    <Button
      class="w-full gap-2"
      onclick={() => deployTemplate(template.id)}
      disabled={!$activeTeam?.team}
    >
      <Plus class="h-4 w-4" />
      Deploy
    </Button>
  </Card.Footer>
</Card.Root>
