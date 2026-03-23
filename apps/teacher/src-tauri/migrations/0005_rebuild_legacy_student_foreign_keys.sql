-- V1.4 修复 0002_split_students 之后遗留的旧 student 外键引用。
--
-- 0002 中将旧 students 重命名为 students_old，再创建新的 students / student_exams。
-- SQLite 在表重命名时会把旧表上的外键引用改写为 students_old，导致后续
-- answer_sheets / score_summary / cheat_logs 等表仍然依赖一个已经被 DROP 的表。
--
-- 这里通过重建相关表，把 student_id 外键重新指回当前 students(id)，并在复制
-- 数据时把历史旧 student_id（实际是旧 students.id，也就是当前 student_exams.id）
-- 映射回新的 students.id。

CREATE TABLE answer_sheets__new (
    id                TEXT    PRIMARY KEY,
    student_exam_id   TEXT    REFERENCES student_exams(id),
    student_id        TEXT    NOT NULL REFERENCES students(id),
    exam_id           TEXT    NOT NULL REFERENCES exams(id),
    question_id       TEXT    NOT NULL REFERENCES questions(id),
    answer            TEXT,
    revision          INTEGER NOT NULL DEFAULT 1,
    answer_updated_at INTEGER,
    received_at       INTEGER,
    is_correct        INTEGER,
    score             INTEGER,
    synced_at         INTEGER,
    UNIQUE(student_id, question_id)
);

INSERT OR REPLACE INTO answer_sheets__new(
    id,
    student_exam_id,
    student_id,
    exam_id,
    question_id,
    answer,
    revision,
    answer_updated_at,
    received_at,
    is_correct,
    score,
    synced_at
)
SELECT
    a.id,
    COALESCE(
        a.student_exam_id,
        (SELECT se.id FROM student_exams se WHERE se.id = a.student_id LIMIT 1),
        (SELECT se.id FROM student_exams se WHERE se.exam_id = a.exam_id AND se.student_id = a.student_id LIMIT 1)
    ) AS student_exam_id,
    COALESCE(
        (SELECT se.student_id FROM student_exams se WHERE se.id = a.student_exam_id LIMIT 1),
        (SELECT se.student_id FROM student_exams se WHERE se.id = a.student_id LIMIT 1),
        (SELECT s.id FROM students s WHERE s.id = a.student_id LIMIT 1)
    ) AS student_id,
    a.exam_id,
    a.question_id,
    a.answer,
    COALESCE(a.revision, 1),
    a.answer_updated_at,
    a.received_at,
    a.is_correct,
    a.score,
    a.synced_at
FROM answer_sheets a
WHERE COALESCE(
    (SELECT se.student_id FROM student_exams se WHERE se.id = a.student_exam_id LIMIT 1),
    (SELECT se.student_id FROM student_exams se WHERE se.id = a.student_id LIMIT 1),
    (SELECT s.id FROM students s WHERE s.id = a.student_id LIMIT 1)
) IS NOT NULL;

DROP TABLE answer_sheets;
ALTER TABLE answer_sheets__new RENAME TO answer_sheets;

CREATE INDEX IF NOT EXISTS idx_answer_sheets_student_exam_id ON answer_sheets(student_exam_id);
CREATE INDEX IF NOT EXISTS idx_answer_sheets_exam_id ON answer_sheets(exam_id);

CREATE TABLE score_summary__new (
    id          TEXT    PRIMARY KEY,
    exam_id     TEXT    NOT NULL REFERENCES exams(id),
    student_id  TEXT    NOT NULL REFERENCES students(id),
    total_score INTEGER,
    is_passed   INTEGER,
    graded_at   INTEGER,
    UNIQUE(exam_id, student_id)
);

INSERT OR REPLACE INTO score_summary__new(
    id,
    exam_id,
    student_id,
    total_score,
    is_passed,
    graded_at
)
SELECT
    ss.id,
    ss.exam_id,
    COALESCE(
        (SELECT se.student_id FROM student_exams se WHERE se.id = ss.student_id LIMIT 1),
        (SELECT s.id FROM students s WHERE s.id = ss.student_id LIMIT 1)
    ) AS student_id,
    ss.total_score,
    ss.is_passed,
    ss.graded_at
FROM score_summary ss
WHERE COALESCE(
    (SELECT se.student_id FROM student_exams se WHERE se.id = ss.student_id LIMIT 1),
    (SELECT s.id FROM students s WHERE s.id = ss.student_id LIMIT 1)
) IS NOT NULL;

DROP TABLE score_summary;
ALTER TABLE score_summary__new RENAME TO score_summary;

CREATE TABLE cheat_logs__new (
    id          TEXT    PRIMARY KEY,
    student_id  TEXT    NOT NULL REFERENCES students(id),
    event_type  TEXT    NOT NULL,
    detail      TEXT,
    occurred_at INTEGER NOT NULL
);

INSERT INTO cheat_logs__new(
    id,
    student_id,
    event_type,
    detail,
    occurred_at
)
SELECT
    cl.id,
    COALESCE(
        (SELECT se.student_id FROM student_exams se WHERE se.id = cl.student_id LIMIT 1),
        (SELECT s.id FROM students s WHERE s.id = cl.student_id LIMIT 1)
    ) AS student_id,
    cl.event_type,
    cl.detail,
    cl.occurred_at
FROM cheat_logs cl
WHERE COALESCE(
    (SELECT se.student_id FROM student_exams se WHERE se.id = cl.student_id LIMIT 1),
    (SELECT s.id FROM students s WHERE s.id = cl.student_id LIMIT 1)
) IS NOT NULL;

DROP TABLE cheat_logs;
ALTER TABLE cheat_logs__new RENAME TO cheat_logs;