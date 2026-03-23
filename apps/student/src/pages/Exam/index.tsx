import { useState } from "react";
import { Button } from "antd";

import AnswerList from "../../components/ExamContent/AnswerList";
import ImageList from "../../components/ExamContent/ImageList";
import { useExamStore } from "@/store/examStore";
import { sendAnswerSync } from "@/services/examRuntimeService";

export default function ExamPage() {
  const questions = useExamStore((s) => s.questions);
  const currentSession = useExamStore((s) => s.currentSession);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [selectedAnswers, setSelectedAnswers] = useState<Record<string, number>>(
    {},
  );

  if (questions.length === 0) {
    return (
      <main className="h-full border border-slate-200 bg-white p-6 shadow-sm flex items-center justify-center">
        <p className="text-slate-600">暂无试卷数据，请等待教师重新分发。</p>
      </main>
    );
  }

  const currentQuestion = questions[currentIndex];

  const handleSelectAnswer = (optionIndex: number) => {
    setSelectedAnswers((prev) => ({
      ...prev,
      [currentQuestion.id]: optionIndex,
    }));

    const option = currentQuestion.options[optionIndex];
    const answerValue = option?.key || `${optionIndex + 1}`;

    if (!currentSession) {
      return;
    }

    void sendAnswerSync(
      currentSession.examId,
      currentSession.studentId,
      currentQuestion.id,
      answerValue,
    ).catch((error) => {
      console.error("[ExamPage] 答案同步失败", error);
    });
  };

  return (
    <main className="h-full border border-slate-200 bg-white p-6 shadow-sm">
      <div className="space-y-5">
        <header className="space-y-2 border-b border-slate-100 pb-4">
          <p className="text-sm font-medium text-sky-600">
            第 {currentIndex + 1} 题 / 共 {questions.length} 题
          </p>
          <h1 className="text-lg font-semibold text-slate-900">
            {currentQuestion.content}
          </h1>
        </header>

        <ImageList images={currentQuestion.images} />

        <section className="space-y-2">
          <h2 className="text-base font-medium text-slate-800">答案列表</h2>
          <AnswerList
            options={currentQuestion.options}
            selectedOption={selectedAnswers[currentQuestion.id] ?? null}
            onSelect={handleSelectAnswer}
          />
        </section>

        <footer className="flex items-center justify-between border-t border-slate-100 pt-4">
          <Button
            onClick={() => setCurrentIndex((prev) => Math.max(prev - 1, 0))}
            disabled={currentIndex === 0}
            className="border-slate-300 text-slate-700 hover:border-slate-400 hover:bg-slate-50"
          >
            上一题
          </Button>
          <Button
            type="primary"
            onClick={() =>
              setCurrentIndex((prev) =>
                  Math.min(prev + 1, questions.length - 1),
              )
            }
            disabled={currentIndex === questions.length - 1}
            className="bg-sky-600 hover:bg-sky-700"
          >
            下一题
          </Button>
        </footer>
      </div>
    </main>
  );
}
