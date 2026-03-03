export type ExamStatus = "draft" | "published" | "active" | "paused" | "finished";

export interface Exam {
  id: string;
  title: string;
  status: ExamStatus;
}

export type QuestionType = "single" | "multi" | "judge" | "fill" | "essay";

export interface Question {
  id: string;
  examId: string;
  type: QuestionType;
  content: string;
  score: number;
}
