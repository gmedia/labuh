import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Labuh",
  description: "Lightweight PaaS Platform for container deployment",
  head: [["link", { rel: "icon", href: "/favicon.png" }]],
  themeConfig: {
    logo: "/logo.png",
    nav: [
      { text: "Home", link: "/" },
      { text: "Guide", link: "/guide/introduction" },
      { text: "API", link: "/api/reference" },
    ],
    sidebar: [
      {
        text: "Getting Started",
        items: [
          { text: "Introduction", link: "/guide/introduction" },
          { text: "Installation", link: "/guide/installation" },
          { text: "Architecture", link: "/guide/architecture" },
        ],
      },
      {
        text: "Deployment",
        items: [
          { text: "Docker Stacks", link: "/guide/stacks" },
          { text: "Domains & SSL", link: "/guide/domains" },
          { text: "Webhooks", link: "/guide/webhooks" },
        ],
      },
      {
        text: "Hardware",
        items: [
          { text: "Optimization & Edge", link: "/guide/optimization" },
          { text: "Private Registries", link: "/guide/registries" },
        ],
      },
    ],
    socialLinks: [
      { icon: "github", link: "https://github.com/HasanH47/labuh" },
    ],
    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2026-present Labuh Team",
    },
  },
});
