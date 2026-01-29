# Stacks

**Stacks** adalah inti dari Labuh. Ini memungkinkan Anda mendeploy aplikasi yang terdiri dari satu atau banyak container menggunakan format standard `docker-compose.yml`.

## Mengimpor Stack

Anda dapat membuat stack baru di dashboard Labuh:

1. Klik menu **Stacks**.
2. Klik tombol **Create Stack**.
3. Berikan nama untuk stack Anda.
4. Masukkan (paste) konten `docker-compose.yml` Anda.
5. Labuh akan memproses file tersebut dan membuat container-container terkait.

## Pengelolaan Stack

Di halaman detail stack, Anda dapat:

- **Start/Stop/Restart**: Mengontrol seluruh stack sekaligus.
- **Redeploy**: Menarik image terbaru dan membuat ulang container.
- **Log Viewer**: Melihat log gabungan dari seluruh container dalam stack tersebut.
- **Update Compose**: Mengubah konfigurasi YAML dan melakukan sinkronisasi otomatis.

## Network & Volume

- **Networking**: Labuh secara otomatis menyatukan semua stack ke dalam satu Docker network internal agar mereka bisa saling berkomunikasi menggunakan nama service.
- **Volumes**: Labuh mendukung penulisan volume lokal maupun named volume sesuai standar Docker.
