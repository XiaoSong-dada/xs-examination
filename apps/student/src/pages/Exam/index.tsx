import { useState } from "react";
import { Button } from "antd";

import AnswerList from "../../components/ExamContent/AnswerList";
import ImageList from "../../components/ExamContent/ImageList";

type TempQuestion = {
  id: number;
  title: string;
  images: string[];
  options: string[];
};

const tempQuestions: TempQuestion[] = [
  {
    id: 1,
    title: "下列哪项最符合图示场景中的操作规范？",
    images: [
      "https://images.unsplash.com/photo-1460925895917-afdab827c52f?auto=format&fit=crop&w=900&q=80",
      "https://images.unsplash.com/photo-1454165804606-c3d57bc86b40?auto=format&fit=crop&w=900&q=80",
    ],
    options: [
      "先核对信息，再执行关键步骤",
      "直接按经验操作，后续再补记录",
      "只要系统可提交即可，无需复查",
      "由同学代替本人完成确认",
    ],
  },
  {
    id: 2,
    title: "根据图片内容，最合理的处理方式是？",
    images: [
      "https://images.unsplash.com/photo-1517430816045-df4b7de11d1d?auto=format&fit=crop&w=900&q=80",
    ],
    options: [
      "按流程依次检查并记录结果",
      "遇到异常时跳过当前步骤",
      "仅凭主观判断给出结论",
      "无需提交过程记录",
    ],
  },
  {
    id: 3,
    title: "下图对应题目中，哪项描述是正确的？",
    images: [
      "https://images.unsplash.com/photo-1553877522-43269d4ea984?auto=format&fit=crop&w=900&q=80",
      "https://images.unsplash.com/photo-1521737604893-d14cc237f11d?auto=format&fit=crop&w=900&q=80",
      "https://images.unsplash.com/photo-1498050108023-c5249f4df085?auto=format&fit=crop&w=900&q=80",
    ],
    options: [
      "图片仅作装饰，不影响判断",
      "应结合图示信息与题干共同判断",
      "只看第一张图即可确定答案",
      "所有选项均不成立",
    ],
  },
];

export default function ExamPage() {
  const [currentIndex, setCurrentIndex] = useState(0);
  const [selectedAnswers, setSelectedAnswers] = useState<Record<number, number>>(
    {},
  );

  const currentQuestion = tempQuestions[currentIndex];

  const handleSelectAnswer = (optionIndex: number) => {
    setSelectedAnswers((prev) => ({
      ...prev,
      [currentQuestion.id]: optionIndex,
    }));
  };

  return (
    <main className="h-full border border-slate-200 bg-white p-6 shadow-sm">
      <div className="space-y-5">
        <header className="space-y-2 border-b border-slate-100 pb-4">
          <p className="text-sm font-medium text-sky-600">
            第 {currentIndex + 1} 题 / 共 {tempQuestions.length} 题
          </p>
          <h1 className="text-lg font-semibold text-slate-900">
            {currentQuestion.title}
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
                Math.min(prev + 1, tempQuestions.length - 1),
              )
            }
            disabled={currentIndex === tempQuestions.length - 1}
            className="bg-sky-600 hover:bg-sky-700"
          >
            下一题
          </Button>
        </footer>
      </div>
    </main>
  );
}
