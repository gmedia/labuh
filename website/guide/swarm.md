# Swarm & Cluster

Labuh v0.4.4 menghadirkan dukungan native untuk **Docker Swarm**, memungkinkan Anda mengelola cluster dari banyak server (node) langsung dari satu dashboard.

## Ringkasan Swarm

Docker Swarm mengubah kumpulan host Docker menjadi satu server virtual tunggal. Labuh terintegrasi mulus dengan Swarm untuk menyediakan:

- **Deployment Multi-Node**: Deploy stack di berbagai server sekaligus.
- **High Availability**: Service dapat direplikasi di berbagai node untuk ketahanan sistem.
- **Manajemen Terpusat**: Kelola Manager dan Worker dari satu antarmuka pengguna.

## Inisialisasi Swarm

Jika server Anda belum menjadi bagian dari Swarm, Anda dapat menginisialisasinya langsung dari Labuh:

1. Pergi ke menu sidebar **Nodes**.
2. Klik **Initialize Swarm**.
3. Labuh akan mengkonfigurasi server saat ini sebagai node **Manager** utama.

## Manajemen Node

Setelah Swarm aktif, halaman **Nodes** menjadi pusat kendali cluster Anda.

### Menambahkan Node

Untuk menambahkan lebih banyak server ke cluster Anda:

1. Klik **Join Token** pada node Manager.
2. Salin **Worker Token** (atau Manager Token).
3. Di server baru (yang harus sudah terinstal Docker), jalankan perintah `docker swarm join` yang diberikan ATAU gunakan antarmuka Labuh jika Labuh juga terinstal di sana.

### Promosi & Demosi Node

- **Promote to Manager**: Memberikan hak administratif pada node Worker.
- **Demote to Worker**: Mencabut hak administratif (berguna untuk pemeliharaan).

Anda dapat melakukan tindakan ini dengan mengklik tombol **...** (Opsi) pada setiap kartu node.

### Terminal & Monitoring

- **Node Terminal**: Akses shell host server langsung dari browser menggunakan ikon terminal di setiap kartu node. Sangat berguna untuk maintenance server jarak jauh.
- **Resource Monitoring**: Pantau kapasitas CPU, RAM, dan Disk dari setiap node secara real-time.

## Visualisasi Jaringan

Labuh menyediakan **Network Visualizer** yang powerful untuk membantu Anda memahami topologi jaringan di dalam cluster.

- **Interactive Graph**: Lihat bagaimana container terhubung ke network dan container lainnya.
- **Real-time Status**: Warna pada node menunjukkan kesehatan service Anda.

Akses visualizer melalui menu **Networks** di sidebar dashboard.
