import { api, type TemplateResponse } from "$lib/api";
import { toast } from "svelte-sonner";

export class TemplateController {
  templates = $state<TemplateResponse[]>([]);
  loading = $state(true);
  searchQuery = $state("");
  showAddDialog = $state(false);
  importMode = $state<"json" | "url">("url");
  urlInput = $state("");
  jsonInput = $state("");
  adding = $state(false);

  async init() {
    await this.loadTemplates();
  }

  async loadTemplates() {
    this.loading = true;
    const result = await api.templates.list();
    if (result.data) {
      this.templates = result.data;
    }
    this.loading = false;
  }

  get filteredTemplates() {
    if (!this.searchQuery) return this.templates;
    const q = this.searchQuery.toLowerCase();
    return this.templates.filter(
      (t) =>
        t.name.toLowerCase().includes(q) ||
        t.description.toLowerCase().includes(q),
    );
  }

  async addTemplate() {
    this.adding = true;
    try {
      if (this.importMode === "url") {
        if (!this.urlInput) return;
        const result = await api.templates.import(this.urlInput);
        if (result.data) {
          toast.success("Template imported successfully");
          this.showAddDialog = false;
          this.urlInput = "";
          await this.loadTemplates();
        } else {
          toast.error(
            result.message || result.error || "Failed to import template",
          );
        }
      } else {
        if (!this.jsonInput) return;
        const template = JSON.parse(this.jsonInput);
        const result = await api.templates.create(template);
        if (!result.error) {
          toast.success("Template added successfully");
          this.showAddDialog = false;
          this.jsonInput = "";
          await this.loadTemplates();
        } else {
          toast.error(
            result.message || result.error || "Failed to add template",
          );
        }
      }
    } catch (e: any) {
      toast.error(e.message || "Error processing request");
    } finally {
      this.adding = false;
    }
  }
}
