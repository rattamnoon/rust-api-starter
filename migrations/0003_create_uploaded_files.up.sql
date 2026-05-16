CREATE TABLE uploaded_files (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    sub_folder text NOT NULL,
    original_filename text NOT NULL,
    stored_filename text NOT NULL,
    content_type text NULL,
    size_bytes bigint NOT NULL,
    storage_path text NOT NULL,
    uploaded_by uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (sub_folder, stored_filename)
);

CREATE INDEX uploaded_files_uploaded_by_idx ON uploaded_files (uploaded_by);
CREATE INDEX uploaded_files_sub_folder_idx ON uploaded_files (sub_folder);
