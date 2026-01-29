---
layout: home

hero:
  name: "Labuh"
  text: "Lightweight PaaS Platform"
  tagline: "High-performance container management for modern servers and edge devices."
  image:
    src: /logo.png
    alt: Labuh Logo
  actions:
    - theme: brand
      text: Get Started
      link: /guide/introduction
    - theme: alt
      text: View on GitHub
      link: https://github.com/HasanH47/labuh

features:
  - title: Built for Performance
    details: Written in Rust for maximum speed and minimum memory footprint. Fast on x86_64, efficient on ARM.
    icon: 🚀
  - title: Universal Compatibility
    details: Runs anywhere Docker does. From powerful cloud servers to small edge devices and ARM STBs.
    icon: 🌍
  - title: Docker Compose Native
    details: Simply paste your docker-compose.yml and let Labuh handle the rest. Automatic networking and volume setup.
    icon: 📦
  - title: Zero-Config SSL
    details: Built-in Caddy integration provides automatic SSL via Let's Encrypt for all your custom domains.
    icon: 🔒
  - title: Integrated Dashboard
    details: A beautiful, modern UI to manage your stacks, containers, and logs without needing extra services.
    icon: 📊
  - title: Deployment Webhooks
    details: Easy-to-use API endpoints for automated deployments. Perfect for your existing CI/CD pipelines.
    icon: �
---

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #00c6ff 30%, #0072ff);
}
</style>
