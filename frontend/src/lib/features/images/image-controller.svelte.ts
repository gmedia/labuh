import { api, type Image } from "$lib/api";
import { activeTeam } from "$lib/stores";
import { toast } from "svelte-sonner";
import { get } from "svelte/store";

export class ImageController {
  images = $state<Image[]>([]);
  loading = $state(true);
  imageUrl = $state("");
  pulling = $state(false);
  actionLoading = $state<string | null>(null);

  async init() {
    await this.loadImages();
  }

  async loadImages() {
    this.loading = true;
    const result = await api.images.list();
    if (result.data) {
      this.images = result.data;
    }
    this.loading = false;
  }

  async pullImage() {
    const team = get(activeTeam)?.team;
    if (!this.imageUrl || !team) return;
    this.pulling = true;
    const result = await api.images.pull(this.imageUrl, team.id);
    if (result.data) {
      toast.success(`Image ${this.imageUrl} pulled successfully`);
      this.imageUrl = "";
      await this.loadImages();
    } else {
      toast.error(result.message || result.error || "Failed to pull image");
    }
    this.pulling = false;
  }

  async removeImage(id: string) {
    if (!confirm("Are you sure you want to delete this image?")) return;
    this.actionLoading = id;
    const result = await api.images.remove(id);
    if (result.error) {
      toast.error(result.message || result.error || "Failed to remove image");
    } else {
      toast.success("Image removed");
      await this.loadImages();
    }
    this.actionLoading = null;
  }
}
