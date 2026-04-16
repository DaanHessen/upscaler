# Implementation Plan - Supabase SaaS Integration (Fast-Path)

We are migrating the Gemini Upscaler to a production-ready SaaS environment. This plan is optimized for **minimal latency** and **wallet protection** by avoiding the "Double Hop" upload trap.

## User Review Required

> [!IMPORTANT]
> **Fast-Path Storage**: The user will upload images **directly to Axum** (Multipart). 
> 1. Axum processes the image in-memory.
> 2. Axum uploads the result to Supabase Storage.
> 3. Axum returns a **Signed URL** to the user.
> *Optional*: The original image is uploaded to Supabase asynchronously in the background so it doesn't block the upscale.

> [!CAUTION]
> **Anti-Abuse (Authenticated Only)**: Every request MUST have a valid `Authorization: Bearer <JWT>` header. Anonymous calls are strictly blocked to protect your API quota and storage from botnets.

## Proposed Changes

### [Component] Environment & Dependencies

#### [MODIFY] [.env](file:///c:/Users/daanh/Documents/AA-GEMINI-UPSCALER-RUST-BACKEND/.env)
- Add `SUPABASE_JWT_SECRET`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`, and `DATABASE_URL`.

#### [MODIFY] [Cargo.toml](file:///c:/Users/daanh/Documents/AA-GEMINI-UPSCALER-RUST-BACKEND/Cargo.toml)
- Add `aws-sdk-s3`, `aws-config`, `jsonwebtoken`, and `sqlx` (with `runtime-tokio-rustls`, `postgres`).

### [Component] Services

#### [NEW] [auth.rs](file:///c:/Users/daanh/Documents/AA-GEMINI-UPSCALER-RUST-BACKEND/src/auth.rs)
- Implement `JwtAuth` extractor for Axum.
- Validates Supabase JWTs and returns the `user_id`.

#### [NEW] [storage.rs](file:///c:/Users/daanh/Documents/AA-GEMINI-UPSCALER-RUST-BACKEND/src/storage.rs)
- Implement S3 client using `force_path_style(true)` and the specific Supabase endpoint.
- Functions: `upload_object(bucket, path, bytes)` and `generate_signed_url(bucket, path)`.

#### [NEW] [db.rs](file:///c:/Users/daanh/Documents/AA-GEMINI-UPSCALER-RUST-BACKEND/src/db.rs)
- Simple SQLx wrapper to record `upscales` (id, user_id, style, input_path, output_path, timestamp).

### [Component] Orchestration

#### [MODIFY] [main.rs](file:///c:/Users/daanh/Documents/AA-GEMINI-UPSCALER-RUST-BACKEND/src/main.rs)
- Refactor `upscale_handler` to:
  1. Authenticate user.
  2. Launch async task to archive the original image in Supabase `originals/`.
  3. Preprocess and Upscale (main task).
  4. Upload result to Supabase `processed/`.
  5. Save record to DB.
  6. Return JSON with the Signed URL.

## Open Questions

1.  **Bucket Name**: Do you have a preferred bucket name (e.g., `upscales`)?
2.  **DB Schema**: Should I create a migration file to set up the `upscales` table in your Supabase DB?

## Verification Plan

### Automated Tests
- Integration test using `reqwest` to simulate a signed-in request and verify that the resulting Signed URL actually yields the image.

### Manual Verification
- Confirm that the `originals/` and `processed/` folders in your Supabase Dashboard are populating correctly.
