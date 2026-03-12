-- V1.2 设备列表表
-- 仅创建基础字段：id/ip/name
CREATE TABLE IF NOT EXISTS devices (
    id   TEXT PRIMARY KEY,
    ip   TEXT NOT NULL,
    name TEXT NOT NULL
);
