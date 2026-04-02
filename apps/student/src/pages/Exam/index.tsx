import { useEffect, useMemo, useState } from "react";
import { Button } from "antd";

import AnswerCard from "@/components/ExamContent/AnswerCard";
import AnswerList from "../../components/ExamContent/AnswerList";
import ImageList from "../../components/ExamContent/ImageList";
import { useFileHooks } from "@/hooks/useFileHooks";
import { useExamStore } from "@/store/examStore";
import { getCurrentSessionAnswers, sendAnswerSync } from "@/services/examRuntimeService";

/**
 * 渲染学生端答题页面，并维护当前题目、已答状态与答案同步动作。
 * @param props 页面组件不接收外部参数。
 * @returns 带答题卡与题目内容区的考试页面。
 */
export default function ExamPage() {
  const questions = useExamStore((s) => s.questions);
  const currentSession = useExamStore((s) => s.currentSession);
  const { resolveImagePreviews } = useFileHooks();
  const [currentIndex, setCurrentIndex] = useState(0);
  const [selectedAnswers, setSelectedAnswers] = useState<Record<string, number>>(
    {},
  );
  const [answerCardCollapsed, setAnswerCardCollapsed] = useState(false);
  const [imagePreviewMap, setImagePreviewMap] = useState<Record<string, string>>({});

  useEffect(() => {
    if (!currentSession || questions.length === 0) {
      return;
    }

    void getCurrentSessionAnswers()
      .then((answers) => {
        const restored: Record<string, number> = {};
        for (const item of answers) {
          const question = questions.find((q) => q.id === item.questionId);
          if (!question) {
            continue;
          }

          const optionIndex = question.options.findIndex(
            (option, index) =>
              option.key === item.answer ||
              option.text === item.answer ||
              `${index + 1}` === item.answer,
          );

          if (optionIndex >= 0) {
            restored[question.id] = optionIndex;
          }
        }

        setSelectedAnswers(restored);
      })
      .catch((error) => {
        console.error("[ExamPage] 读取本地答案失败", error);
      });
  }, [currentSession, questions]);

  useEffect(() => {
    setCurrentIndex((prev) => {
      if (questions.length === 0) {
        return 0;
      }

      return Math.min(prev, questions.length - 1);
    });
  }, [questions]);

  useEffect(() => {
    const imagePaths = questions.flatMap((question) => [
      ...(question.images ?? []),
      ...question.options.flatMap((option) => option.imagePaths ?? []),
    ]);

    if (imagePaths.length === 0) {
      setImagePreviewMap({});
      return;
    }

    let cancelled = false;
    void (async () => {
      try {
        const previews = await resolveImagePreviews(imagePaths);
        if (!cancelled) {
          setImagePreviewMap(previews);
        }
      } catch (_error) {
        if (!cancelled) {
          setImagePreviewMap({});
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [questions, resolveImagePreviews]);

  if (questions.length === 0) {
    return (
      <main className="h-full border border-slate-200 bg-white p-6 shadow-sm flex items-center justify-center">
        <p className="text-slate-600">暂无试卷数据，请等待教师重新分发。</p>
      </main>
    );
  }

  const displayQuestions = useMemo(
    () =>
      questions.map((question) => ({
        ...question,
        images: (question.images ?? []).map(
          (path) => imagePreviewMap[path] ?? path,
        ),
        options: question.options.map((option) => ({
          ...option,
          imagePaths: (option.imagePaths ?? []).map(
            (path) => imagePreviewMap[path] ?? path,
          ),
        })),
      })),
    [questions, imagePreviewMap],
  );

  const currentQuestion = displayQuestions[currentIndex];
  const answeredQuestionIds = Object.keys(selectedAnswers);

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
    <div className="flex h-full gap-4 p-4 lg:p-6">
      <AnswerCard
        questions={questions}
        currentIndex={currentIndex}
        answeredQuestionIds={answeredQuestionIds}
        collapsed={answerCardCollapsed}
        onQuestionSelect={setCurrentIndex}
        onToggle={() => setAnswerCardCollapsed((prev) => !prev)}
      />

      <main className="flex h-full min-w-0 flex-1 flex-col rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex-1 space-y-5 overflow-y-auto pr-1">
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
        </div>

        <footer className="mt-5 flex items-center justify-between border-t border-slate-100 pt-4">
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
      </main>
    </div>
  );
}
