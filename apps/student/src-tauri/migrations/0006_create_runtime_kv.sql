-- V1.5 创建运行时状态表
-- 用于崩溃恢复当前题号、心跳时间、最后同步时间等轻量状态
CREATE TABLE runtime_kv (
    key         TEXT    PRIMARY KEY,
    value       TEXT    NOT NULL,
    updated_at  INTEGER NOT NULL
);