import { useCallback, useEffect, useMemo, useState } from "react";
import * as XLSX from "xlsx";

import { useAllExamList } from "@/hooks/useExam";
import { getExamById, updateExam } from "@/services/examService";
import { getStudentDeviceConnectionStatusByExamId } from "@/services/studentService";
import type { StudentDeviceConnectionStatusItem } from "@/types/main";

export interface ReportTableItem {
  id: string;
  name: string;
  deviceIp: string;
  answerProgress: number;
  score: number;
}

/**
 * 成绩报告页面 Hook：负责考试切换、表格数据与导出。
 */
export function useReport() {
  const { exams, loading: examLoading } = useAllExamList();
  const [students, setStudents] = useState<StudentDeviceConnectionStatusItem[]>([]);
  const [tableLoading, setTableLoading] = useState(false);

  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [exporting, setExporting] = useState(false);

  useEffect(() => {
    if (!selectedExamId && exams.length > 0) {
      setSelectedExamId(exams[0].id);
    }
  }, [exams, selectedExamId]);

  const refresh = useCallback(async () => {
    if (!selectedExamId) {
      setStudents([]);
      return;
    }

    setTableLoading(true);
    try {
      const list = await getStudentDeviceConnectionStatusByExamId(selectedExamId);
      setStudents(list);
    } finally {
      setTableLoading(false);
    }
  }, [selectedExamId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const examOptions = useMemo(
    () => exams.map((exam) => ({ label: exam.title, value: exam.id })),
    [exams],
  );

  const selectedExamTitle = useMemo(() => {
    const exam = exams.find((item) => item.id === selectedExamId);
    return exam?.title ?? "未命名考试";
  }, [exams, selectedExamId]);

  const tableData = useMemo<ReportTableItem[]>(
    () =>
      students.map((student: StudentDeviceConnectionStatusItem) => ({
        id: student.student_id,
        name: student.student_name,
        deviceIp: student.ip_addr ?? "-",
        answerProgress: student.progress_percent ?? 0,
        score: 0,
      })),
    [students],
  );

  const exportReport = useCallback(async () => {
    setExporting(true);
    try {
      const rows = tableData.map((item) => ({
        学生姓名: item.name,
        学生设备IP: item.deviceIp,
        答题进度: `${item.answerProgress}%`,
        分值: item.score,
      }));

      const worksheet = XLSX.utils.json_to_sheet(rows);
      const workbook = XLSX.utils.book_new();
      XLSX.utils.book_append_sheet(workbook, worksheet, "成绩报告");

      const fileName = `${selectedExamTitle}-成绩报告.xlsx`;
      XLSX.writeFile(workbook, fileName);

      if (selectedExamId) {
        const detail = await getExamById(selectedExamId);
        if (detail.status !== "archived") {
          await updateExam({
            id: detail.id,
            title: detail.title,
            description: detail.description,
            start_time: detail.start_time,
            end_time: detail.end_time,
            pass_score: detail.pass_score,
            status: "archived",
            shuffle_questions: detail.shuffle_questions,
            shuffle_options: detail.shuffle_options,
          });
        }
      }

      return true;
    } catch (error) {
      console.error("[useReport] 导出成绩失败", error);
      return false;
    } finally {
      setExporting(false);
    }
  }, [selectedExamId, selectedExamTitle, tableData]);

  return {
    selectedExamId,
    setSelectedExamId,
    examOptions,
    examLoading,
    tableData,
    tableLoading,
    exporting,
    exportReport,
    refresh,
  } as const;
}
