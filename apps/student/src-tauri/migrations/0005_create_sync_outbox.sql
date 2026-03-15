-- V1.4 创建出站同步队列表
-- 用于缓存答案同步、状态上报、作弊告警和主动交卷事件
CREATE TABLE sync_outbox (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id     TEXT    NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
    event_type     TEXT    NOT NULL,
    aggregate_id   TEXT,
    payload        BLOB    NOT NULL,
    status         TEXT    NOT NULL DEFAULT 'pending',
    retry_count    INTEGER NOT NULL DEFAULT 0,
    next_retry_at  INTEGER,
    last_error     TEXT,
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL
);

CREATE INDEX idx_sync_outbox_session_status ON sync_outbox(session_id, status);
CREATE INDEX idx_sync_outbox_next_retry_at ON sync_outbox(next_retry_at);