import { Image } from "antd";

import type { ExamQuestionOption } from "@/types/exam";

type AnswerListProps = {
  options: ExamQuestionOption[];
  selectedOption: number | null;
  onSelect: (index: number) => void;
};

/**
 * 渲染可点击作答的选项列表，并支持选项图片预览交互。
 * @param props 选项数组、当前选中项及点击回调。
 * @returns 返回选项按钮列表。
 */
export default function AnswerList({
  options,
  selectedOption,
  onSelect,
}: AnswerListProps) {
  return (
    <div className="space-y-3">
      {options.map((option, index) => {
        const isSelected = selectedOption === index;
        return (
          <button
            key={`${option.key}`}
            type="button"
            onClick={() => onSelect(index)}
            className={`flex w-full items-start gap-3 rounded-md border px-4 py-3 text-left transition ${
              isSelected
                ? "border-sky-500 bg-sky-50 text-sky-700"
                : "border-slate-200 bg-white text-slate-700 hover:border-sky-300"
            }`}
          >
            <span className="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-current text-xs font-semibold">
              {option.key || `${index + 1}`}
            </span>
            <span className="flex-1 space-y-2">
              <span className="block text-sm leading-6">{option.text}</span>
              {Array.isArray(option.imagePaths) && option.imagePaths.length > 0 ? (
                <span className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                  {option.imagePaths.map((imagePath, imageIndex) => (
                    <Image
                      key={`${option.key}-${imagePath}-${imageIndex}`}
                      src={imagePath}
                      alt={`选项${option.key}图片${imageIndex + 1}`}
                      width="100%"
                      height={112}
                      style={{ objectFit: "cover" }}
                    />
                  ))}
                </span>
              ) : null}
            </span>
          </button>
        );
      })}
    </div>
  );
}