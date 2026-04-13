# Testing the S3 Sync Backend Locally

Vaelorium's S3 backend is a thin translation layer on top of `aws-sdk-s3`. The engine logic is exercised by 11+ filesystem-backend integration scenarios in `src-tauri/tests/sync_integration.rs`; the S3 backend's unit tests cover config parsing, prefix handling, etag cleanup, and error classification. What the unit tests *can't* cover is real network behavior and authentication.

For end-to-end validation against a real S3-compatible service, use Minio locally.

## Option 1 — Minio (recommended)

```bash
# Run Minio in Docker with default creds
docker run -d --name minio \
  -p 9000:9000 -p 9001:9001 \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  minio/minio server /data --console-address ":9001"

# Create a bucket
docker exec minio mc alias set local http://localhost:9000 minioadmin minioadmin
docker exec minio mc mb local/vaelorium-test
```

Then in the Vaelorium desktop app → Settings → Sync → Enable sync, pick **S3-compatible**, and fill in:

| Field | Value |
|---|---|
| Endpoint URL | `http://localhost:9000` |
| Region | `us-east-1` |
| Bucket | `vaelorium-test` |
| Access key ID | `minioadmin` |
| Secret access key | `minioadmin` |
| Prefix | `vaelorium` (optional) |

Set a passphrase, click **Enable sync for this Tome**. Edit a page; the sidebar pill should briefly show "Syncing…" then return to "Synced". Use `mc ls local/vaelorium-test/vaelorium` to see the encrypted snapshot + journal blobs.

## Option 2 — Cloudflare R2

1. Create a bucket in the R2 dashboard.
2. Create an R2 API token with read+write access to that bucket.
3. In Vaelorium:
   - **Endpoint URL:** `https://<account-id>.r2.cloudflarestorage.com`
   - **Region:** `auto`
   - **Bucket:** your bucket name
   - **Access key ID / Secret access key:** from the R2 token

## Option 3 — AWS S3

1. Create a bucket and an IAM user with bucket-specific policy.
2. In Vaelorium:
   - **Endpoint URL:** leave empty
   - **Region:** matching your bucket's region (`us-east-1`, `eu-west-2`, etc.)
   - **Bucket:** your bucket name
   - **Access key ID / Secret access key:** from IAM

## What to verify

- **Initial upload** — after enabling, check that `<prefix>/snapshots/` and `<prefix>/journal/` populate over time.
- **Cross-device pickup** — enable sync on a second device with the same bucket + passphrase; edit on one, observe on the other after a sync cycle (~10s of nudge debounce or 5min poll).
- **Conflict surfacing** — edit the same field offline on two devices; after both sync, the `ConflictResolver` banner should appear on the affected page.
- **Auth failure** — enter wrong credentials; enabling sync should fail with "authentication failed" rather than a raw AWS stack trace.
- **Unreachable endpoint** — set endpoint to `http://invalid.example/`; enabling should fail with "endpoint unreachable".

## Known gaps

- **`If-Match` CAS semantics** vary across S3-compatible providers. AWS S3 and Minio support it; some older/self-hosted gateways don't. The snapshot pointer update relies on it; if your provider doesn't support conditional writes, two devices syncing simultaneously could race on snapshot promotion. Revisit in Phase 5 polish.
- **Binary image files don't sync.** Only SQLite-backed data syncs via the op journal. Images live outside the DB and would need a separate file-sync layer (probably M8 collaboration territory).
- **No OS keychain integration yet.** Credentials are stored plaintext in `sync_config.backend_config` inside the local `.tome`. The `.tome` itself is trusted (the user opens it); broader credential hygiene lands in Phase 5+.
