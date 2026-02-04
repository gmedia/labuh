# Introduction

**Labuh** (dari bahasa Melayu/Indonesia: _berlabuh_ = to dock/berth) adalah platform PaaS (Platform as a Service) modern, aman, dan ringan yang dirancang untuk mendeploy, menarik, dan membangun container image di berbagai infrastruktur.

## ⚓ Filosofi Labuh

Nama **Labuh** tidak sekadar dipilih karena singkatannya yang teknis, melainkan sebuah filosofi: **Tempat di mana kode Anda akhirnya berlabuh.**

Bagi seorang pengembang, kode adalah hasil penjelajahan kreatif yang panjang. Setelah melewati badai _debugging_ dan pengujian di samudra _development_, kode tersebut membutuhkan pelabuhan yang tenang untuk bersandar—sebuah tempat yang stabil agar ia bisa mulai melayani penggunanya.

Labuh hadir sebagai **dermaga modern** bagi aplikasi Anda. Kami percaya bahwa mendeploy kode seharusnya semudah melempar jangkar: presisi, kokoh, dan tanpa drama.

---

## Mengapa Labuh?

Labuh diciptakan untuk menjembatani celah antara Docker Compose manual yang membosankan dan platform PaaS enterprise yang terlalu berat dan kompleks.

- **Performa Tinggi**: Ditulis dalam Rust untuk mendapatkan kecepatan native dengan penggunaan resource yang sangat efisien.
- **Universal**: Berjalan mulus di server enterprise (x86_64) maupun perangkat hemat energi (ARMv8/ARM64).
- **Dashboard Terintegrasi**: Pengelolaan visual lengkap yang disajikan langsung dari binary backend—tanpa perlu runtime tambahan seperti Node.js di server.
- **Full Control**: Anda memegang kendali penuh atas data dan container Anda, tanpa ketergantungan pada cloud provider tertentu.

## Fitur Unggulan

- **Docker Swarm Mode**: Dukungan native untuk clustering, multi-node scaling, dan high availability.
- **Universal Multi-Arch**: Berjalan di infrastruktur x86_64 dan ARM64 (Raspberry Pi, TV Box STB, dll).
- **Git Integration & Build**: Deploy langsung dari repository Git dan bangun image secara otomatis.
- **Manajemen Stack**: Deploy aplikasi multi-container menggunakan format standar `docker-compose.yml`.
- **Terminal Interaktif**: Akses shell container DAN host server langsung dari dashboard.
- **Smart Ingress**: Integrasi Caddy otomatis dengan dukungan **Cloudflare Tunnels** dan **Multi-Provider DNS**.
- **Network Visualization**: Visualisasi topologi jaringan container dan cluster secara interaktif.
- **Monitoring & Resource Tracking**: Pantau metrik historis CPU/RAM dan log container secara real-time.
- **Webhook Automation**: Integrasikan dengan CI/CD untuk deployment otomatis saat image baru tersedia.
- **App Templates**: Galeri aplikasi populer yang siap deploy dalam hitungan detik.
