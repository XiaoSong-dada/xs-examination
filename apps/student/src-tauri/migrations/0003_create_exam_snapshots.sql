-- V1.2 创建考试快照表
-- 只落学生端展示与作答必需的试卷快照，不保存标准答案
CREATE TABLE exam_snapshots (
    session_id         TEXT    PRIMARY KEY REFERENCES exam_sessions(id) ON DELETE CASCADE,
    exam_meta          BLOB    NOT NULL,
    questions_payload  BLOB    NOT NULL,
    downloaded_at      INTEGER NOT NULL,
    expires_at         INTEGER,
    updated_at         INTEGER NOT NULL
);