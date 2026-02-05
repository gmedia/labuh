# Optimization & Edge Devices

Labuh dirancang untuk berjalan kencang di server standar, namun efisiensinya benar-benar bersinar saat dijalankan di perangkat dengan resource terbatas (_Edge Devices_) seperti TV Box (STB), Raspberry Pi, atau VPS paket hemat.

## Karakteristik Resource Rendah

Berbeda dengan platform PaaS lain yang membutuhkan runtime Node.js atau Python yang berat di sisi server untuk dashboard, Labuh menggunakan pendekatan:

- **Rust Backend**: Native performance dengan jejak RAM yang sangat kecil (biasanya < 50MB).
- **Embedded Frontend**: Dashboard disajikan sebagai aset statis. Browser pengguna yang melakukan beban kerja rendering, bukan server Anda.

## Optimalisasi untuk STB / ARM

Jika Anda menggunakan perangkat seperti TV Box dengan Armbian, berikut adalah tips optimalisasi:

### 1. Penanganan Konfigurasi Caddy

Pada beberapa filesystem di perangkat ARM, mount Docker terhadap file yang belum ada bisa memicu pembuatan folder kosong. Labuh v0.4.4+ telah dilengkapi fitur **Auto-Repair**:

- Secara otomatis mendeteksi dan memperbaiki `Caddyfile` yang rusak atau terdeteksi sebagai direktori.
- Menginisialisasi konfigurasi default secara mandiri jika file tidak ditemukan.

### 2. Swap Space

Untuk perangkat dengan RAM 1GB - 2GB, sangat disarankan untuk mengaktifkan **Swap** (minimal 1GB) agar proses build atau pull image besar tidak menyebabkan sistem hang.

### 3. Database SQLite

SQLite adalah pilihan tepat untuk perangkat edge karena ia berupa file tunggal dan tidak membutuhkan proses background tambahan seperti PostgreSQL atau MySQL, yang menghemat siklus CPU dan RAM.

## Dukungan x86_64

Meskipun sangat efisien di ARM, Labuh tetap memberikan performa maksimal di server x86_64 standar. Kecepatan Rust dalam menangani API request dan Docker interaction membuatnya sangat responsif bahkan di bawah beban kerja yang tinggi.
