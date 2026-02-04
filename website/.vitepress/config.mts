import { defineConfig } from "vitepress";
import { withMermaid } from "vitepress-plugin-mermaid";

export default withMermaid(
  defineConfig({
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
          text: "Swarm & Clusters",
          items: [
            { text: "Swarm Overview", link: "/guide/swarm" },
            { text: "Node Management", link: "/guide/swarm#node-management" },
            {
              text: "Network Visualization",
              link: "/guide/swarm#network-visualization",
            },
          ],
        },
        {
          text: "Core Features",
          items: [
            { text: "Docker Stacks", link: "/guide/stacks" },
            { text: "App Templates", link: "/guide/templates" },
            { text: "Containers & Images", link: "/guide/containers-images" },
            { text: "Webhooks (CI/CD)", link: "/guide/webhooks" },
          ],
        },
        {
          text: "Management",
          items: [
            { text: "Teams & Access", link: "/guide/teams" },
            { text: "Domains & SSL", link: "/guide/domains" },
            { text: "Private Registries", link: "/guide/registries" },
            { text: "Resource Monitoring", link: "/guide/resources" },
          ],
        },
        {
          text: "Hardware",
          items: [{ text: "Optimization & Edge", link: "/guide/optimization" }],
        },
      ],
      socialLinks: [
        { icon: "github", link: "https://github.com/gmedia/labuh" },
      ],
      footer: {
        message: "Released under the MIT License.",
        copyright: "Copyright © 2026-present Labuh Team",
      },
    },
  }),
);
