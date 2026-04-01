CREATE TABLE IF NOT EXISTS question_bank_items (
    id                  TEXT    PRIMARY KEY,
    type                TEXT    NOT NULL,
    content             TEXT    NOT NULL,
    content_image_paths TEXT,
    options             TEXT,
    answer              TEXT    NOT NULL,
    score               INTEGER NOT NULL DEFAULT 0,
    explanation         TEXT,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_question_bank_items_updated_at
    ON question_bank_items(updated_at DESC);