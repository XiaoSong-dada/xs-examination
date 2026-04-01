CREATE TABLE IF NOT EXISTS exam_question_assets (
    id                 TEXT    PRIMARY KEY,
    session_id         TEXT    NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
    exam_id            TEXT    NOT NULL,
    question_id        TEXT    NOT NULL,
    scope              TEXT    NOT NULL,
    asset_local_path   TEXT    NOT NULL,
    source_archive_path TEXT,
    checksum           TEXT,
    created_at         INTEGER NOT NULL,
    updated_at         INTEGER NOT NULL
);

CREATE INDEX idx_exam_question_assets_session_id
    ON exam_question_assets(session_id);

CREATE INDEX idx_exam_question_assets_exam_question
    ON exam_question_assets(exam_id, question_id);
