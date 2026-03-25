-- V1.5 将 answer_sheets 幂等冲突键收敛到 student_exam_id + question_id。
--
-- 目标：避免跨考试仅凭 student_id + question_id 发生误覆盖，
-- 并与 ANSWER_SYNC 基于 student_exam_id 的聚合口径一致。

CREATE TABLE answer_sheets__v2 (
    id                TEXT    PRIMARY KEY,
    student_exam_id   TEXT    NOT NULL REFERENCES student_exams(id) ON DELETE CASCADE,
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
    UNIQUE(student_exam_id, question_id)
);

INSERT OR REPLACE INTO answer_sheets__v2(
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
        (
            SELECT se.id
            FROM student_exams se
            WHERE se.exam_id = a.exam_id
              AND se.student_id = a.student_id
            LIMIT 1
        )
    ) AS student_exam_id,
    COALESCE(
        (
            SELECT se.student_id
            FROM student_exams se
            WHERE se.id = a.student_exam_id
            LIMIT 1
        ),
        a.student_id
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
    a.student_exam_id,
    (
        SELECT se.id
        FROM student_exams se
        WHERE se.exam_id = a.exam_id
          AND se.student_id = a.student_id
        LIMIT 1
    )
) IS NOT NULL;

DROP TABLE answer_sheets;
ALTER TABLE answer_sheets__v2 RENAME TO answer_sheets;

CREATE INDEX IF NOT EXISTS idx_answer_sheets_student_exam_id ON answer_sheets(student_exam_id);
CREATE INDEX IF NOT EXISTS idx_answer_sheets_exam_id ON answer_sheets(exam_id);
CREATE INDEX IF NOT EXISTS idx_answer_sheets_student_id ON answer_sheets(student_id);
