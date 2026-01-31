<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { theme } from '$lib/stores';
</script>

<Card.Root class="h-fit">
  <Card.Header>
    <Card.Title>Appearance</Card.Title>
    <Card.Description>Customize the look and feel</Card.Description>
  </Card.Header>
  <Card.Content>
    <div class="flex items-center justify-between">
      <div>
        <p class="font-medium">Dark Mode</p>
        <p class="text-sm text-muted-foreground">Toggle between light and dark theme</p>
      </div>
      <Button variant="outline" onclick={() => theme.toggle()}>
        {$theme === 'dark' ? 'Switch to Light' : 'Switch to Dark'}
      </Button>
    </div>
  </Card.Content>
</Card.Root>
