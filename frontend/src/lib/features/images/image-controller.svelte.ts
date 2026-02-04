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

  // UI States
  showDeleteConfirm = $state(false);
  imageToDelete = $state<string | null>(null);

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

    // Validation: Enforce image:tag
    if (!this.imageUrl.includes(":")) {
      toast.error("Please specify a tag (e.g., nginx:latest)");
      return;
    }

    this.pulling = true;
    toast.info(`Starting to pull ${this.imageUrl}...`);

    try {
      const result = await api.images.pull(this.imageUrl, team.id);
      if (result.data) {
        toast.success(`Image ${this.imageUrl} pulled successfully`);
        this.imageUrl = "";
        await this.loadImages();
      } else {
        toast.error(result.message || result.error || "Failed to pull image");
      }
    } catch (err) {
      toast.error("Network error during image pull");
    } finally {
      this.pulling = false;
    }
  }

  requestDelete(id: string) {
    this.imageToDelete = id;
    this.showDeleteConfirm = true;
  }

  async confirmDelete() {
    if (!this.imageToDelete) return;
    const id = this.imageToDelete;
    this.actionLoading = id;
    this.showDeleteConfirm = false;

    try {
      const result = await api.images.remove(id);
      if (result.error) {
        toast.error(result.message || result.error || "Failed to remove image");
      } else {
        toast.success("Image removed");
        await this.loadImages();
      }
    } catch (err) {
      toast.error("Network error during image removal");
    } finally {
      this.actionLoading = null;
      this.imageToDelete = null;
    }
  }
}
