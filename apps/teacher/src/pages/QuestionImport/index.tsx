import { Button, Select, Table, message } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useMemo, useRef, useState } from "react";

import { useAllExamList } from "@/hooks/useExam";
import { useTableHeight } from "@/hooks/useTableHeight";
import { getQuestionListByExamId } from "@/services/questionService";
import type { QuestionListItem } from "@/types/main";

/**
 * 教师端题库导入页面。
 *
 * @returns 返回考试筛选与题目列表。
 */
export function QuestionImportPage() {
  const { exams, loading: examLoading } = useAllExamList();
  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [questionLoading, setQuestionLoading] = useState(false);
  const [questions, setQuestions] = useState<QuestionListItem[]>([]);

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef);

  const examOptions = useMemo(
    () => exams.map((exam) => ({ label: exam.title, value: exam.id })),
    [exams],
  );

  useEffect(() => {
    if (!selectedExamId && exams.length > 0) {
      setSelectedExamId(exams[0].id);
    }
  }, [exams, selectedExamId]);

  useEffect(() => {
    const fetchQuestions = async () => {
      if (!selectedExamId) {
        setQuestions([]);
        return;
      }

      setQuestionLoading(true);
      try {
        const result = await getQuestionListByExamId({ exam_id: selectedExamId });
        setQuestions(result);
      } catch (error) {
        console.error("获取题目列表失败", error);
        message.error("获取题目列表失败");
        setQuestions([]);
      } finally {
        setQuestionLoading(false);
      }
    };

    void fetchQuestions();
  }, [selectedExamId]);

  const columns: ColumnsType<QuestionListItem> = [
    {
      title: "序号",
      dataIndex: "seq",
      key: "seq",
      width: 80,
    },
    {
      title: "题型",
      dataIndex: "type",
      key: "type",
      width: 120,
    },
    {
      title: "题目内容",
      dataIndex: "content",
      key: "content",
    },
    {
      title: "分值",
      dataIndex: "score",
      key: "score",
      width: 100,
    },
  ];

  return (
    <div className="space-y-4 h-full">
      <div
        ref={containerRef}
        className="bg-white rounded-lg border border-gray-200 p-4 h-full"
      >
        <div
          ref={toolbarRef}
          className="bg-white rounded-lg flex flex-col gap-5 pb-4 w-full"
        >
          <div className="flex gap-2">
            <Select
              className="w-full max-w-md"
              placeholder="请选择考试"
              value={selectedExamId}
              loading={examLoading}
              options={examOptions}
              onChange={setSelectedExamId}
            />
            <Button
              type="primary"
              onClick={() => message.info("导入功能开发中")}
            >
              导入题库
            </Button>
          </div>
        </div>

        <Table<QuestionListItem>
          rowKey="id"
          loading={questionLoading}
          dataSource={questions}
          columns={columns}
          pagination={false}
          scroll={{ y: tableHeight }}
        />
      </div>
    </div>
  );
}
