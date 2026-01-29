# Domains & SSL

Labuh mempermudah Anda untuk mengekspos aplikasi ke internet menggunakan domain kustom dengan keamanan HTTPS otomatis.

## Bagaimana Ini Bekerja?

Labuh secara otomatis mengelola containter **Caddy** di port 80 dan 443 server Anda. Caddy bertindak sebagai reverse proxy cerdas yang menghubungkan domain Anda ke container yang tepat di dalam Labuh.

## Menambahkan Domain

1. Pastikan domain Anda sudah diarahkan (A Record atau CNAME) ke alamat IP server Anda.
2. Di Dashboard Labuh, masuk ke menu Stack atau Container yang ingin diekspos.
3. Tambahkan domain baru (misal: `app.anda.com`).
4. Labuh akan memerintahkan Caddy untuk meminta sertifikat SSL dari Let's Encrypt secara otomatis.

## Verifikasi DNS

Labuh menyediakan fitur verifikasi DNS sederhana di dashboard untuk memastikan domain Anda sudah diarahkan ke IP yang benar sebelum Caddy mencoba mengambil sertifikat SSL.

## Basic Auth

Anda dapat memproteksi rute/domain tertentu dengan **Basic Auth** (Username & Password) langsung dari dashboard. Fitur ini sangat berguna untuk memproteksi dashboard admin atau aplikasi internal yang tidak memiliki sistem login sendiri.
