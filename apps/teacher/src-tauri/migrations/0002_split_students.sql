-- V1.1 学生表拆分迁移
-- 注意：sqlx 会自动包裹事务，此处不要手动 BEGIN/COMMIT
--
-- 策略：
--   1. 将旧 students（每次考试一条）重命名为 students_old
--   2. 新建持久化 students（每个真实学生一条），合并旧表中与人相关的字段
--      + 补充 created_at / updated_at 时间戳字段
--   3. 新建 student_exams（每次参考一条），保留旧表中与考试相关的字段
--   4. 迁移数据：旧 students → 新 students & student_exams
--   5. 删除临时旧表 students_old

-- Step 1: 保留旧数据，重命名旧表
ALTER TABLE students RENAME TO students_old;

-- Step 2: 新建持久化学生信息表（字段合并：student_no/name 来自旧表，created_at/updated_at 为新增）
CREATE TABLE IF NOT EXISTS students (
    id          TEXT    PRIMARY KEY,
    student_no  TEXT    NOT NULL UNIQUE,    -- 学号（全局唯一，代表同一个真实学生）
    name        TEXT    NOT NULL,
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s','now')*1000),
    updated_at  INTEGER NOT NULL DEFAULT (strftime('%s','now')*1000)
);

-- Step 3: 新建学生参加考试记录表（字段合并：exam_id/ip_addr/status/join_time/submit_time 来自旧表）
CREATE TABLE IF NOT EXISTS student_exams (
    id          TEXT    PRIMARY KEY,
    student_id  TEXT    NOT NULL REFERENCES students(id),
    exam_id     TEXT    NOT NULL REFERENCES exams(id),
    ip_addr     TEXT,
    status      TEXT    NOT NULL DEFAULT 'waiting',   -- waiting|active|submitted|offline|forced
    join_time   INTEGER,
    submit_time INTEGER,
    UNIQUE(exam_id, student_id)
);

-- Step 4a: 将旧 students 中每个不同学号插入新 students
-- 同一学号多次参考时取 rowid 最小（最早）那条的 name
INSERT OR IGNORE INTO students(id, student_no, name, created_at, updated_at)
SELECT
    s.student_no                                          AS id,
    s.student_no,
    s.name,
    COALESCE(s.join_time, (strftime('%s','now')*1000))   AS created_at,
    COALESCE(s.join_time, (strftime('%s','now')*1000))   AS updated_at
FROM students_old s
WHERE s.rowid = (
    SELECT MIN(s2.rowid) FROM students_old s2 WHERE s2.student_no = s.student_no
);

-- Step 4b: 将旧 students 的所有参考记录迁移到 student_exams
-- student_id 使用新 students.id（即 student_no），沿用旧 students.id 作为参考记录 id
INSERT INTO student_exams(id, student_id, exam_id, ip_addr, status, join_time, submit_time)
SELECT
    o.id,
    o.student_no   AS student_id,
    o.exam_id,
    o.ip_addr,
    o.status,
    o.join_time,
    o.submit_time
FROM students_old o;

-- Step 5: 删除旧表
DROP TABLE students_old;
