# Webhooks

Labuh mendukung otomatisasi deployment (CI/CD) melalui **Webhooks**. Ini memungkinkan server CI Anda (seperti GitHub Actions) atau Docker Registry (seperti Docker Hub) untuk memberi tahu Labuh saat image baru telah dibangun.

## Cara Kerja Webhook

Setiap Stack di Labuh memiliki sebuah **Webhook Token** unik. Saat request POST dikirim ke URL webhook tersebut, Labuh akan:

1. Memverifikasi token.
2. Menarik (pull) image terbaru untuk seluruh service dalam stack tersebut.
3. Melakukan restart (_Rolling Restart_) pada container yang imagenya berubah.

## Endpoint Webhook

URL webhook untuk stack Anda memiliki format:

```text
POST http://<alamat-labuh>:3000/api/webhooks/deploy/<stack-id>/<token>
```

## Integrasi GitHub Actions

Anda dapat menambahkan langkah berikut pada workflow GitHub Actions Anda untuk memicu deploy otomatis setiap kali build image selesai:

```yaml
- name: Trigger Labuh Deploy
  run: |
    curl -X POST http://server.anda.com:3000/api/webhooks/deploy/${{ secrets.LABUH_STACK_ID }}/${{ secrets.LABUH_TOKEN }}
```

## Keamanan

- **Token-based**: Setiap stack memiliki token acak 32 karakter yang sulit ditebak.
- **Regenerasi**: Anda dapat membuat ulang (regenerate) token sewaktu-waktu dari dashboard jika merasa token lama telah bocor.
