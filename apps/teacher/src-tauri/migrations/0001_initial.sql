-- V1.0 MVP 初始化：教师端全量表结构

-- 考试表
CREATE TABLE IF NOT EXISTS exams (
    id                TEXT    PRIMARY KEY,
    title             TEXT    NOT NULL,
    description       TEXT,
    start_time        INTEGER,
    end_time          INTEGER,
    pass_score        INTEGER NOT NULL,
    status            TEXT    NOT NULL DEFAULT 'draft',
    shuffle_questions INTEGER          DEFAULT 0,
    shuffle_options   INTEGER          DEFAULT 0,
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL
);

-- 题目表
CREATE TABLE IF NOT EXISTS questions (
    id          TEXT    PRIMARY KEY,
    exam_id     TEXT    NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    seq         INTEGER NOT NULL,
    type        TEXT    NOT NULL,
    content     TEXT    NOT NULL,
    options     TEXT,
    answer      TEXT    NOT NULL,
    score       INTEGER NOT NULL,
    explanation TEXT
);

-- 学生表（每次考试独立注册）
CREATE TABLE IF NOT EXISTS students (
    id          TEXT    PRIMARY KEY,
    exam_id     TEXT    NOT NULL REFERENCES exams(id),
    student_no  TEXT    NOT NULL,
    name        TEXT    NOT NULL,
    ip_addr     TEXT,
    status      TEXT    NOT NULL DEFAULT 'waiting',
    join_time   INTEGER,
    submit_time INTEGER,
    UNIQUE(exam_id, student_no)
);

-- 答卷表
CREATE TABLE IF NOT EXISTS answer_sheets (
    id          TEXT    PRIMARY KEY,
    student_id  TEXT    NOT NULL REFERENCES students(id),
    exam_id     TEXT    NOT NULL REFERENCES exams(id),
    question_id TEXT    NOT NULL REFERENCES questions(id),
    answer      TEXT,
    is_correct  INTEGER,
    score       INTEGER,
    synced_at   INTEGER,
    UNIQUE(student_id, question_id)
);

-- 成绩汇总表
CREATE TABLE IF NOT EXISTS score_summary (
    id          TEXT    PRIMARY KEY,
    exam_id     TEXT    NOT NULL REFERENCES exams(id),
    student_id  TEXT    NOT NULL REFERENCES students(id),
    total_score INTEGER,
    is_passed   INTEGER,
    graded_at   INTEGER,
    UNIQUE(exam_id, student_id)
);

-- 防作弊告警日志
CREATE TABLE IF NOT EXISTS cheat_logs (
    id          TEXT    PRIMARY KEY,
    student_id  TEXT    NOT NULL REFERENCES students(id),
    event_type  TEXT    NOT NULL,
    detail      TEXT,
    occurred_at INTEGER NOT NULL
);
