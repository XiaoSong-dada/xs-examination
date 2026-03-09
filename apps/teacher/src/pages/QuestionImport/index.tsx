import { Button, Modal, Select, Table, message } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useMemo, useRef, useState } from "react";

import { useAllExamList } from "@/hooks/useExam";
import { useQuestion } from "@/hooks/useQuestion";
import { useTableHeight } from "@/hooks/useTableHeight";
import type { Question } from "@/types/main";
import { parseXlsxFile, type XlsxRow } from "@/utils/xlsx";

/**
 * 教师端题库导入页面。
 *
 * @returns 返回考试筛选与题目列表。
 */
export function QuestionImportPage() {
  const { exams, loading: examLoading } = useAllExamList();
  const {
    questions,
    loading: questionLoading,
    fetchQuestionsByExamId,
    importQuestionsByExamId,
  } = useQuestion();

  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [importing, setImporting] = useState(false);
  const [importModalVisible, setImportModalVisible] = useState(false);
  const [importQuestions, setImportQuestions] = useState<Question[]>([]);

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
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
      try {
        await fetchQuestionsByExamId(selectedExamId);
      } catch {
        message.error("获取题目列表失败");
      }
    };

    void fetchQuestions();
  }, [fetchQuestionsByExamId, selectedExamId]);

  const normalizeQuestionRow = (row: XlsxRow, index: number, examId: string): Question => {
    const toString = (value: unknown): string => (value == null ? "" : String(value).trim());
    const toNumber = (value: unknown, fallback: number): number => {
      const num = Number(value);
      return Number.isFinite(num) ? num : fallback;
    };

    const seqRaw = row.seq ?? row["序号"] ?? index + 1;
    const typeRaw = row.type ?? row["题型"] ?? "";
    const contentRaw = row.content ?? row["题目内容"] ?? row["题干"] ?? "";
    const optionsRaw = row.options ?? row["选项"];
    const answerRaw = row.answer ?? row["答案"] ?? "";
    const scoreRaw = row.score ?? row["分值"] ?? 0;
    const explanationRaw = row.explanation ?? row["解析"];

    return {
      id: `import-${Date.now()}-${index}`,
      exam_id: examId,
      seq: toNumber(seqRaw, index + 1),
      type: toString(typeRaw),
      content: toString(contentRaw),
      options: toString(optionsRaw) || undefined,
      answer: toString(answerRaw),
      score: toNumber(scoreRaw, 0),
      explanation: toString(explanationRaw) || undefined,
    };
  };

  const handleSelectImportFile = () => {
    if (!selectedExamId) {
      message.warning("请先选择考试");
      return;
    }
    fileInputRef.current?.click();
  };

  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = event.target.files?.[0];
    event.target.value = "";

    if (!selectedFile || !selectedExamId) {
      return;
    }

    setImporting(true);
    try {
      const rows = await parseXlsxFile<XlsxRow>(selectedFile, {
        raw: false,
        defval: "",
      });

      const normalized = rows
        .map((row, index) => normalizeQuestionRow(row, index, selectedExamId))
        .filter((item) => item.content && item.type && item.answer);

      if (normalized.length === 0) {
        message.warning("未解析到有效题目，请检查 Excel 列名或内容");
        return;
      }

      setImportQuestions(normalized);
      setImportModalVisible(true);
    } catch (error) {
      console.error("解析题库文件失败", error);
      message.error("解析题库文件失败，请检查文件格式");
    } finally {
      setImporting(false);
    }
  };

  const handleConfirmImport = async () => {
    if (importQuestions.length === 0) {
      message.warning("没有可导入的题目数据");
      return;
    }

    if (!selectedExamId) {
      message.warning("请先选择考试");
      return;
    }

    setImporting(true);
    try {
      const inserted = await importQuestionsByExamId(selectedExamId, importQuestions);
      setImportModalVisible(false);
      setImportQuestions([]);
      message.success(`成功导入 ${inserted.length} 条题目`);
    } catch (error) {
      console.error("导入题库失败", error);
      message.error("导入题库失败，请稍后重试");
    } finally {
      setImporting(false);
    }
  };

  const columns: ColumnsType<Question> = [
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

  const importColumns: ColumnsType<Question> = [
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
      title: "答案",
      dataIndex: "answer",
      key: "answer",
      width: 160,
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
      <input
        ref={fileInputRef}
        type="file"
        accept=".xlsx,.xls"
        className="hidden"
        onChange={handleFileChange}
      />

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
              loading={importing}
              onClick={handleSelectImportFile}
            >
              导入题库
            </Button>
          </div>
        </div>

        <Table<Question>
          rowKey="id"
          loading={questionLoading}
          dataSource={questions}
          columns={columns}
          pagination={false}
          scroll={{ y: tableHeight }}
        />
      </div>

      <Modal
        title="确认导入题库"
        width={920}
        open={importModalVisible}
        onCancel={() => setImportModalVisible(false)}
        onOk={() => void handleConfirmImport()}
        okText="确认导入"
        cancelText="取消"
        confirmLoading={importing}
      >
        <Table<Question>
          rowKey="id"
          dataSource={importQuestions}
          columns={importColumns}
          pagination={false}
          scroll={{ y: 360 }}
        />
      </Modal>
    </div>
  );
}
