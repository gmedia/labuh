<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { auth } from '$lib/stores';
  import type { SettingsController } from '../settings-controller.svelte';

  let { ctrl = $bindable() } = $props<{ ctrl: SettingsController }>();
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>Profile</Card.Title>
    <Card.Description>Your account information</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    <div class="space-y-2">
      <Label for="email">Email</Label>
      <Input id="email" value={$auth.user?.email || ''} disabled />
    </div>
    <div class="space-y-2">
      <Label for="name">Name</Label>
      <Input id="name" bind:value={ctrl.name} placeholder="Your name" />
    </div>
    <div class="space-y-2">
      <Label>Role</Label>
      <p class="text-sm text-muted-foreground capitalize">{$auth.user?.role}</p>
    </div>
  </Card.Content>
  <Card.Footer>
    <Button onclick={() => ctrl.saveProfile()}>Save Changes</Button>
  </Card.Footer>
</Card.Root>
