import { useCallback, useEffect, useMemo, useState } from "react";
import * as XLSX from "xlsx";

import { useAllExamList } from "@/hooks/useExam";
import { useExamStudents } from "@/hooks/useExamStudents";
import type { StudentListItem } from "@/types/main";

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
  const {
    students,
    loading: studentLoading,
    fetchStudentsByExamId,
  } = useExamStudents();

  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [exporting, setExporting] = useState(false);

  useEffect(() => {
    if (!selectedExamId && exams.length > 0) {
      setSelectedExamId(exams[0].id);
    }
  }, [exams, selectedExamId]);

  const refresh = useCallback(async () => {
    await fetchStudentsByExamId(selectedExamId);
  }, [fetchStudentsByExamId, selectedExamId]);

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
      students.map((student: StudentListItem) => ({
        id: student.id,
        name: student.name,
        deviceIp: "-",
        answerProgress: 0,
        score: 0,
      })),
    [students],
  );

  const exportReport = useCallback(() => {
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
      return true;
    } catch (error) {
      console.error("[useReport] 导出成绩失败", error);
      return false;
    } finally {
      setExporting(false);
    }
  }, [selectedExamTitle, tableData]);

  return {
    selectedExamId,
    setSelectedExamId,
    examOptions,
    examLoading,
    tableData,
    tableLoading: studentLoading,
    exporting,
    exportReport,
    refresh,
  } as const;
}
