# Change: Local file uploads

## Summary
Added an authenticated upload endpoint that stores uploaded files on the local filesystem under `./uploads`, persists metadata in PostgreSQL, and exposes a public static route at `/static/{sub_folder}/{file}`.

## Motivation
The API needed a simple file-ingest capability for local development and backend workflows without introducing object storage yet, while still keeping file metadata queryable and avoiding filename collisions.

## Affected Flows
- authenticated client uploads a multipart file
- API validates the payload, optional `sub_folder`, and size
- original filename is preserved in metadata, while a UUIDv7-based filename is generated for storage
- file is persisted to local disk in `./uploads/{sub_folder}`
- metadata is persisted to PostgreSQL
- API returns stored file metadata and public URL
- clients can fetch the file from `/static/{sub_folder}/{file}`

## Modules/Services Changed
- `src/modules/uploads`
- `src/shared/state.rs`
- `migrations/0003_create_uploaded_files.up.sql`
- app routing and OpenAPI registration

## Backward Compatibility
This is additive. Existing endpoints and auth flows are unchanged.

## Operational Notes
Uploaded files are runtime artifacts and are ignored by git through `/uploads`. The default upload directory is controlled by `UPLOAD_DIR` and defaults to `./uploads`. Static file resolution now reads DB metadata first and then serves the corresponding file from disk.

## References
- [README.md](/Users/un/Documents/workshop/rust-api-starter/README.md)
- [docs/architecture/data-flow.md](/Users/un/Documents/workshop/rust-api-starter/docs/architecture/data-flow.md)
