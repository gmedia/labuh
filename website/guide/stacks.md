# Stacks

**Stacks** adalah inti dari Labuh. Ini memungkinkan Anda mendeploy aplikasi yang terdiri dari satu atau banyak container menggunakan format standard `docker-compose.yml`.

## Mengimpor Stack

Anda dapat membuat stack baru di dashboard Labuh:

1. Klik menu **Stacks**.
2. Klik tombol **Create Stack**.
3. Berikan nama untuk stack Anda.
4. Masukkan (paste) konten `docker-compose.yml` Anda.
5. Labuh akan memproses file tersebut dan membuat container-container terkait.

## Deployment via Git

Labuh menghadirkan integrasi Git tingkat lanjut:

1. **Deploy dari Git**: Masukkan URL repository (Public atau Private), branch, dan path ke `docker-compose.yml`.
2. **Build dari Dockerfile**: Jika `docker-compose` Anda menggunakan direktif `build:`, Labuh akan melakukan clone dan build image secara otomatis di server.
3. **Log Build Real-time**: Pantau proses build langkah-demi-langkah lewat modal **Build Logs**.
4. **Git Sync**: Lakukan sinkronisasi repository dan redeploy kapanpun source code Anda berubah.

## Pengelolaan & Otomatisasi

Di halaman detail stack, Anda dapat:

- **Start/Stop/Restart**: Mengontrol seluruh lifecycle stack.
- **Redeploy & Rollback**: Menarik image terbaru atau kembali ke versi konfigurasi sebelumnya jika terjadi error.
- **Automation Settings**:
  - **Cron Schedule**: Atur jadwal deployment otomatis (misal: setiap jam atau setiap tengah malam).
  - **Health Checks**: Pantau ketersediaan aplikasi via endpoint HTTP secara berkala.
- **Terminal Exec**: Akses shell interaktif langsung dari dashboard menggunakan xterm.js.
- **Log Viewer**: Streaming log gabungan dari seluruh container dalam stack.
- **Update Compose**: Mengubah konfigurasi YAML secara live.

## Network & Volume

- **Networking**: Labuh secara otomatis menyatukan semua stack ke dalam satu Docker network internal agar mereka bisa saling berkomunikasi menggunakan nama service.
- **Service Replication**: Tentukan `replicas: N` di dalam docker-compose Anda untuk menjalankan multiple instance secara otomatis di seluruh Swarm.
- **Network Visualizer**: Lihat topologi visual stack Anda dengan menekan tombol **Network Map** di detail stack.
