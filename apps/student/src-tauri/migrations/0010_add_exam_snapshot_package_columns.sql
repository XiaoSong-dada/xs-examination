ALTER TABLE exam_snapshots
    ADD COLUMN package_path TEXT;

ALTER TABLE exam_snapshots
    ADD COLUMN package_status TEXT;

ALTER TABLE exam_snapshots
    ADD COLUMN package_batch_id TEXT;

ALTER TABLE exam_snapshots
    ADD COLUMN package_sha256 TEXT;

ALTER TABLE exam_snapshots
    ADD COLUMN package_received_at INTEGER;
