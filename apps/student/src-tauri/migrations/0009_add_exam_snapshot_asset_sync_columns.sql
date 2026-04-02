ALTER TABLE exam_snapshots
    ADD COLUMN assets_sync_status TEXT;

ALTER TABLE exam_snapshots
    ADD COLUMN assets_synced_at INTEGER;
