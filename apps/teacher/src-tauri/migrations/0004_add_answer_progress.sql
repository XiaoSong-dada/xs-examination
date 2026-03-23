-- V1.3 答题同步增强：补充答案接收字段并新增监考进度聚合表

-- answer_sheets 增强字段，用于承接学生端最新答案同步。
ALTER TABLE answer_sheets ADD COLUMN student_exam_id TEXT REFERENCES student_exams(id);
ALTER TABLE answer_sheets ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
ALTER TABLE answer_sheets ADD COLUMN answer_updated_at INTEGER;
ALTER TABLE answer_sheets ADD COLUMN received_at INTEGER;

-- 回填历史数据的 student_exam_id（若存在）。
UPDATE answer_sheets
SET student_exam_id = (
    SELECT se.id
    FROM student_exams se
    WHERE se.exam_id = answer_sheets.exam_id
      AND se.student_id = answer_sheets.student_id
    LIMIT 1
)
WHERE student_exam_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_answer_sheets_student_exam_id ON answer_sheets(student_exam_id);
CREATE INDEX IF NOT EXISTS idx_answer_sheets_exam_id ON answer_sheets(exam_id);

-- 监考答题进度聚合表。
CREATE TABLE IF NOT EXISTS student_exam_progress (
    student_exam_id   TEXT    PRIMARY KEY REFERENCES student_exams(id) ON DELETE CASCADE,
    exam_id           TEXT    NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    student_id        TEXT    NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    answered_count    INTEGER NOT NULL DEFAULT 0,
    total_questions   INTEGER NOT NULL DEFAULT 0,
    progress_percent  INTEGER NOT NULL DEFAULT 0,
    last_question_id  TEXT,
    last_answer_at    INTEGER,
    updated_at        INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_student_exam_progress_exam_id ON student_exam_progress(exam_id);
CREATE INDEX IF NOT EXISTS idx_student_exam_progress_student_id ON student_exam_progress(student_id);
