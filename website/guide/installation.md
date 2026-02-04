# Installation

Labuh dirancang untuk bisa diinstal dengan sangat cepat menggunakan script otomatis (Quick Install) atau secara manual.

## Quick Install (Rekomendasi)

Gunakan perintah satu baris berikut untuk menginstal Labuh secara otomatis di sistem Linux Anda. Script ini akan mendeteksi OS, arsitektur, dan menginstal dependensi yang diperlukan (Docker/containerd).

```bash
curl -fsSL https://raw.githubusercontent.com/gmedia/labuh/main/deploy/quick-install.sh | sudo bash
```

### Apa yang dilakukan script ini?

1. Mengecek arsitektur sistem (x86_64 atau ARM64).
2. Mengecek apakah Docker/containerd sudah terinstal.
3. Mendownload binary Labuh terbaru dari GitHub Release.
4. Menyiapkan user system `labuh`.
5. Membuat service systemd untuk manajemen otomatis.
6. Menyiapkan konfigurasi awal `.env`.

## Persyaratan Sistem

- **OS**: Linux (Ubuntu 22.04+ direkomendasikan).
- **Runtime**: Docker atau containerd.
- **Port**: 3000 (API/Dashboard), 80 & 443 (Caddy Reverse Proxy).

## Konfigurasi Pasca Instalasi

Setelah instalasi selesai, file konfigurasi utama berada di `/opt/labuh/.env`.

```bash
sudo nano /opt/labuh/.env
```

### Variabel Penting:

- `JWT_SECRET`: Kunci rahasia untuk autentikasi (digenerate otomatis).
- `DATABASE_URL`: Alamat database SQLite.
- `FRONTEND_DIR`: Lokasi file dashboard statis.
- `LABUH_PUBLIC_IP`: (Opsional) IP Publik server untuk domain DNS otomatis.

### Inisialisasi Cluster (Opsional)

Jika Anda ingin menjalankan Labuh dalam mode **Swarm** (Clustering):

1. Masuk ke Dashboard.
2. Pergi ke menu **Nodes**.
3. Klik **Initialize Swarm**.
4. Labuh akan mengkonfigurasi node ini sebagai Manager utama.

## Manajemen Service

Gunakan `systemctl` untuk mengelola Labuh:

```bash
sudo systemctl start labuh    # Menjalankan
sudo systemctl stop labuh     # Menghentikan
sudo systemctl restart labuh  # Restart (perlu setelah edit .env)
sudo systemctl status labuh   # Cek status
```

Untuk melihat log secara real-time:

```bash
sudo journalctl -u labuh -f
```
