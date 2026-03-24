import {
  LeftOutlined,
  RightOutlined,
} from "@ant-design/icons";
import type { ExamAnswerCardProps } from "@/types/examCard";

/**
 * 展示当前试卷的答题卡，并提供题目跳转与收起展开能力。
 * @param props 答题卡所需的题目列表、当前题、已答题集合与交互回调。
 * @returns 左侧答题卡侧栏组件。
 */
export default function AnswerCard({
  questions,
  currentIndex,
  answeredQuestionIds,
  collapsed,
  onQuestionSelect,
  onToggle,
}: ExamAnswerCardProps) {
  const answeredQuestionIdSet = new Set(answeredQuestionIds);

  if (collapsed) {
    return (
      <aside className="relative flex h-full w-14 shrink-0 items-center justify-center rounded-2xl border border-gray-300 bg-white shadow-sm">
        <button
          type="button"
          onClick={onToggle}
          className="flex h-full w-full items-center justify-center rounded-2xl text-slate-500 transition hover:bg-slate-200 hover:text-sky-600"
          aria-label="展开答题卡"
          title="展开答题卡"
        >
          <RightOutlined className="text-base" />
        </button>
      </aside>
    );
  }

  return (
    <aside className="flex h-full basis-[30%] flex-col rounded-2xl border border-slate-50 bg-white shadow-sm">
      <div className="flex items-start justify-between gap-3 border-b border-slate-100 px-5 py-4">
        <div className="space-y-1">
          <h2 className="text-base font-semibold text-slate-900">答题卡</h2>
          <p className="text-sm text-slate-500">
            已作答 {answeredQuestionIdSet.size} / {questions.length}
          </p>
        </div>
        <button
          type="button"
          onClick={onToggle}
          className="inline-flex h-9 w-9 items-center justify-center rounded-xl border border-slate-200 text-slate-500 transition hover:border-sky-200 hover:bg-sky-50 hover:text-sky-600"
          aria-label="收起答题卡"
          title="收起答题卡"
        >
          <LeftOutlined className="text-sm" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-5 py-4">
        <div className="mb-4 flex items-center justify-between rounded-2xl border border-slate-200 px-4 py-3 text-sm text-slate-600">
          <span>当前进度</span>
          <span className="font-medium text-slate-900">第 {currentIndex + 1} 题</span>
        </div>

        <div className="flex flex-wrap content-start justify-start gap-3">
          {questions.map((question, index) => {
            const isActive = index === currentIndex;
            const isAnswered = answeredQuestionIdSet.has(question.id);

            return (
              <button
                key={question.id}
                type="button"
                onClick={() => onQuestionSelect(index)}
                className={`flex h-11 w-11 items-center justify-center rounded-lg border text-sm font-semibold transition ${
                  isActive
                    ? "border-sky-500 bg-sky-100 text-sky-700 shadow-sm"
                    : isAnswered
                      ? "border-emerald-200 bg-emerald-500 text-white hover:bg-emerald-600"
                      : "border-slate-200 bg-slate-200 text-slate-600 hover:border-slate-300 hover:bg-slate-300"
                }`}
                aria-label={`跳转到第 ${index + 1} 题`}
                title={`第 ${index + 1} 题`}
              >
                {index + 1}
              </button>
            );
          })}
        </div>
      </div>
    </aside>
  );
}