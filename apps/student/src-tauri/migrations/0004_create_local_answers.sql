-- V1.3 创建本地答案缓存表
-- 同一场考试内按题目唯一，支持断网后继续作答与增量同步
CREATE TABLE local_answers (
    id              TEXT    PRIMARY KEY,
    session_id      TEXT    NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
    question_id     TEXT    NOT NULL,
    answer          TEXT,
    answer_blob     BLOB,
    revision        INTEGER NOT NULL DEFAULT 1,
    sync_status     TEXT    NOT NULL DEFAULT 'pending',
    last_synced_at  INTEGER,
    updated_at      INTEGER NOT NULL,
    UNIQUE(session_id, question_id)
);

CREATE INDEX idx_local_answers_session_id ON local_answers(session_id);
CREATE INDEX idx_local_answers_sync_status ON local_answers(sync_status);