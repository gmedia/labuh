# API Reference

Labuh menyediakan API RESTful untuk manajemen infrastruktur secara terprogram. Semua endpoint diawali dengan `/api` kecuali untuk `health` dan rute publik tertentu.

## Autentikasi

Semua request (kecuali login/register) membutuhkan header `Authorization: Bearer <token>`.

- `POST /api/auth/register`: Mendaftar pengguna baru.
- `POST /api/auth/login`: Login dan mendapatkan token JWT.
- `GET /api/auth/me`: Mendapatkan informasi profil pengguna saat ini.

## Teams

- `GET /api/teams`: List team yang diikuti pengguna.
- `POST /api/teams`: Membuat team baru.
- `DELETE /api/teams/{id}`: Menghapus team.
- `GET /api/teams/{id}/members`: List anggota team.
- `POST /api/teams/{id}/members`: Menambah anggota ke team.
- `PUT /api/teams/{id}/members/{user_id}`: Update role anggota.
- `DELETE /api/teams/{id}/members/{user_id}`: Mengeluarkan anggota dari team.

## Stacks

Inti dari manajemen aplikasi di Labuh.

### Manajemen Stack

- `GET /api/stacks`: List semua stack (opsional filter `?team_id=...`).
- `POST /api/stacks`: Membuat stack baru dari Compose YAML.
- `POST /api/stacks/git`: Membuat stack baru dari repository Git.
- `POST /api/stacks/restore`: Restore stack dari backup.
- `GET /api/stacks/{id}`: Detail stack.
- `DELETE /api/stacks/{id}`: Menghapus stack.

### Operasi Stack

- `POST /api/stacks/{id}/start`: Menjalankan semua container dalam stack.
- `POST /api/stacks/{id}/stop`: Menghentikan semua container dalam stack.
- `POST /api/stacks/{id}/redeploy`: Menarik image terbaru dan membuat ulang container.
- `POST /api/stacks/{id}/build`: Memicu proses build Dockerfile untuk seluruh stack.
- `POST /api/stacks/{id}/rollback`: Kembali ke versi stack sebelumnya.
- `PUT /api/stacks/{id}/compose`: Update konten `docker-compose.yml`.
- `PUT /api/stacks/{id}/automation`: Update cron schedule dan health check.

### Service & Monitoring

- `GET /api/stacks/{id}/containers`: List status container dalam stack.
- `GET /api/stacks/{id}/health`: Status kesehatan stack secara keseluruhan.
- `GET /api/stacks/{id}/logs`: Mengambil log stack (opsional `?tail=...`).
- `GET /api/stacks/{id}/build-logs`: Stream (SSE) log proses build.
- `POST /api/stacks/{id}/services/{service_name}/redeploy`: Redeploy service spesifik.
- `POST /api/stacks/{id}/services/{service_name}/scale`: Mengatur jumlah `replicas` service.
- `POST /api/stacks/{id}/services/{service_name}/build`: Build ulang service spesifik.

## Nodes (Docker Swarm)

- `GET /api/nodes`: List semua node dalam cluster.
- `GET /api/nodes/swarm`: Cek apakah Swarm mode aktif.
- `POST /api/nodes/swarm/init`: Inisialisasi Swarm di node saat ini.
- `POST /api/nodes/swarm/join`: Menghubungkan node ke cluster Swarm yang ada.
- `GET /api/nodes/swarm/tokens`: Mengambil Swarm Join Tokens (Manager & Worker).
- `GET /api/nodes/{id}`: Detail/Inspect node spesifik.
- `GET /api/nodes/terminal`: (WebSocket) Akses terminal host.

## Domains & DNS

- `GET /api/domains`: List semua domain yang terdaftar (filter `?team_id=...`).
- `GET /api/{stack_id}/domains`: List domain milik stack tertentu.
- `POST /api/{stack_id}/domains`: Mendeploy domain baru (Ingress).
- `DELETE /api/{stack_id}/domains/{domain}`: Menghapus rute domain.
- `POST /api/{stack_id}/domains/{domain}/verify`: Menjalankan verifikasi DNS (CNAME/A).
- `PUT /api/{stack_id}/domains/{domain}/dns`: Update record DNS remote (khusus Cloudflare).
- `POST /api/domains/sync`: Sinkronisasi ulang rute Caddy dengan database.

### DNS Config (External Providers)

- `GET /api/teams/{team_id}/dns-configs`: List konfigurasi DNS provider (Cloudflare, dll).
- `POST /api/teams/{team_id}/dns-configs`: Menyimpan konfigurasi API provider.
- `DELETE /api/teams/{team_id}/dns-configs/{provider}`: Menghapus konfigurasi provider.
- `GET /api/teams/{team_id}/dns-configs/{provider}/available-domains`: List zona domain dari provider.
- `GET /api/teams/{team_id}/dns-configs/{provider}/remote-records`: List record DNS yang ada di provider.

## Containers (Standalone)

- `GET /api/containers`: List container (opsional `?all=true`).
- `POST /api/containers/{id}/start`: Start container.
- `POST /api/containers/{id}/stop`: Stop container.
- `POST /api/containers/{id}/restart`: Restart container.
- `DELETE /api/containers/{id}`: Hapus container.
- `GET /api/containers/{id}/logs`: Ambil log container.
- `GET /api/containers/{id}/stats`: Metrik real-time (usage CPU/RAM).
- `GET /api/containers/{id}/exec`: (WebSocket) Akses terminal/shell container.

## Images

- `GET /api/images`: List image lokal.
- `POST /api/images/pull`: Menarik image dari registry.
- `DELETE /api/images/{id}`: Menghapus image.
- `GET /api/images/{id}/inspect`: Detail metadata image.

## Registries

- `GET /api/registries`: List kredensial registry private (filter `?team_id=...`).
- `POST /api/registries`: Menambah kredensial registry baru.
- `DELETE /api/registries/{team_id}/{id}`: Menghapus kredensial.

## Resource Management

- `GET /api/stacks/{stack_id}/limits`: Melihat batas CPU/RAM per service.
- `PUT /api/stacks/{stack_id}/services/{service_name}/limits`: Mengatur batas resource service.
- `GET /api/stacks/{stack_id}/metrics`: Metrik historis untuk seluruh stack.

## Templates

- `GET /api/templates`: Galeri App Templates.
- `POST /api/templates`: Membuat template custom.
- `POST /api/templates/import`: Import template dari URL JSON.
- `GET /api/templates/{id}`: Detail JSON template.
- `DELETE /api/templates/{id}`: Hapus template.

## Networks

- `GET /api/networks/topology`: Mendapatkan data topologi visual jaringan cluster.

## Webhooks

- `POST /api/webhooks/deploy/{stack_id}/{token}`: Memicu redeploy otomatis (opsional `?service=...`).
