-- V1.6 创建教师端地址表（teacher_endpoints）
-- 存储教师端可连接的 endpoint（可包含协议、端口与可选路径），并标记是否为主机教师端
CREATE TABLE teacher_endpoints (
    id          TEXT    PRIMARY KEY,
    endpoint    TEXT    NOT NULL,
    name        TEXT,
    remark      TEXT,
    is_master   INTEGER NOT NULL DEFAULT 0,
    last_seen   INTEGER,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE UNIQUE INDEX idx_teacher_endpoints_endpoint ON teacher_endpoints(endpoint);
