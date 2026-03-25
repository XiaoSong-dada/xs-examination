import type { RuntimeQuestion } from "./main";

export interface ExamAnswerCardProps {
  questions: RuntimeQuestion[];
  currentIndex: number;
  answeredQuestionIds: string[];
  collapsed: boolean;
  onQuestionSelect: (index: number) => void;
  onToggle: () => void;
}