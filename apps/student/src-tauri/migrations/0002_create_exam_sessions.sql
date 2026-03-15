-- V1.1 创建设备分配后的考试会话表
-- 学生端不做登录，按教师端设备分配结果激活一条考试会话
CREATE TABLE exam_sessions (
    id                   TEXT    PRIMARY KEY,
    exam_id              TEXT    NOT NULL,
    student_id           TEXT    NOT NULL,
    student_no           TEXT    NOT NULL,
    student_name         TEXT    NOT NULL,
    assigned_ip_addr     TEXT    NOT NULL,
    assigned_device_name TEXT,
    exam_title           TEXT    NOT NULL,
    status               TEXT    NOT NULL DEFAULT 'waiting',
    assignment_status    TEXT    NOT NULL DEFAULT 'assigned',
    started_at           INTEGER,
    ends_at              INTEGER,
    paper_version        TEXT,
    encryption_nonce     BLOB,
    last_synced_at       INTEGER,
    created_at           INTEGER NOT NULL,
    updated_at           INTEGER NOT NULL
);

CREATE INDEX idx_exam_sessions_exam_id ON exam_sessions(exam_id);
CREATE INDEX idx_exam_sessions_student_id ON exam_sessions(student_id);
CREATE INDEX idx_exam_sessions_assigned_ip_addr ON exam_sessions(assigned_ip_addr);