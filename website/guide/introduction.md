# Introduction

**Labuh** (dari bahasa Melayu/Indonesia: _berlabuh_ = to dock/berth) adalah platform PaaS (Platform as a Service) modern, aman, dan ringan yang dirancang untuk mendeploy, menarik, dan membangun container image di berbagai infrastruktur.

## Mengapa Labuh?

Labuh diciptakan untuk menjembatani celah antara Docker Compose manual yang membosankan dan platform PaaS enterprise yang terlalu berat dan kompleks.

- **Performa Tinggi**: Ditulis dalam Rust untuk mendapatkan kecepatan native dengan penggunaan resource yang sangat efisien.
- **Universal**: Berjalan mulus di server enterprise (x86_64) maupun perangkat hemat energi (ARMv8/ARM64).
- **Dashboard Terintegrasi**: Pengelolaan visual lengkap yang disajikan langsung dari binary backend—tanpa perlu runtime tambahan seperti Node.js di server.
- **Full Control**: Anda memegang kendali penuh atas data dan container Anda, tanpa ketergantungan pada cloud provider tertentu.

## Fitur Unggulan

- **Universal Multi-Arch**: Dukungan native untuk deployment di infrastruktur x86_64 dan ARM64.
- **Manajemen Stack**: Deploy aplikasi multi-container menggunakan format standar `docker-compose.yml`.
- **Reverse Proxy Otomatis**: Integrasi Caddy untuk manajemen rute dan SSL (HTTPS) otomatis.
- **Monitoring & Log**: Pantau kesehatan sistem dan log container secara real-time dari satu dashboard.
- **Webhook Automation**: Integrasikan Labuh dengan alur CI/CD Anda yang sudah ada untuk deployment otomatis.
