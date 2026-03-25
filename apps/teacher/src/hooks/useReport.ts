import { useCallback, useEffect, useMemo, useState } from "react";
import * as XLSX from "xlsx";

import { useAllExamList } from "@/hooks/useExam";
import { getExamById, updateExam } from "@/services/examService";
import {
  calculateStudentScoreSummaryByExamId,
  getStudentDeviceConnectionStatusByExamId,
  getStudentScoreSummaryByExamId,
  resolveReportDownloadPath,
} from "@/services/studentService";
import type { StudentDeviceConnectionStatusItem, StudentScoreSummaryItem } from "@/types/main";

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
  const [scoreSummaryMap, setScoreSummaryMap] = useState<Record<string, number>>({});
  const [scoreSummaryCount, setScoreSummaryCount] = useState(0);
  const [tableLoading, setTableLoading] = useState(false);

  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [exporting, setExporting] = useState(false);
  const [calculating, setCalculating] = useState(false);

  useEffect(() => {
    if (!selectedExamId && exams.length > 0) {
      setSelectedExamId(exams[0].id);
    }
  }, [exams, selectedExamId]);

  const refresh = useCallback(async () => {
    if (!selectedExamId) {
      setStudents([]);
      setScoreSummaryMap({});
      setScoreSummaryCount(0);
      return;
    }

    setTableLoading(true);
    try {
      const [list, scoreSummaryList] = await Promise.all([
        getStudentDeviceConnectionStatusByExamId(selectedExamId),
        getStudentScoreSummaryByExamId(selectedExamId),
      ]);
      setStudents(list);

      const scoreMap = scoreSummaryList.reduce<Record<string, number>>(
        (acc, item: StudentScoreSummaryItem) => {
          acc[item.student_id] = item.total_score;
          return acc;
        },
        {},
      );
      setScoreSummaryMap(scoreMap);
      setScoreSummaryCount(scoreSummaryList.length);
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
        score: scoreSummaryMap[student.student_id] ?? 0,
      })),
    [scoreSummaryMap, students],
  );

  /**
   * 触发成绩统计并刷新表格数据。
   * @returns 统计成功返回 `true`，失败返回 `false`。
   */
  const calculateScores = useCallback(async () => {
    if (!selectedExamId) {
      return false;
    }

    setCalculating(true);
    try {
      await calculateStudentScoreSummaryByExamId(selectedExamId);
      await refresh();
      return true;
    } catch (error) {
      console.error("[useReport] 统计成绩失败", error);
      return false;
    } finally {
      setCalculating(false);
    }
  }, [refresh, selectedExamId]);

  /**
   * 导出成绩报告到本机并返回保存路径。
   * @returns 成功时返回文件绝对路径；失败时返回 `undefined`。
   */
  const exportReport = useCallback(async () => {
    if (!selectedExamId || scoreSummaryCount <= 0) {
      return undefined;
    }

    if (tableData.length <= 0) {
      return undefined;
    }

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
      XLSX.writeFile(workbook, fileName, {
        bookType: "xlsx",
      });
      const downloadPath = await resolveReportDownloadPath(fileName);

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

      return downloadPath;
    } catch (error) {
      console.error("[useReport] 导出成绩失败", error);
      return undefined;
    } finally {
      setExporting(false);
    }
  }, [scoreSummaryCount, selectedExamId, selectedExamTitle, tableData]);

  return {
    selectedExamId,
    setSelectedExamId,
    examOptions,
    examLoading,
    tableData,
    tableLoading,
    calculating,
    exporting,
    calculateScores,
    exportReport,
    scoreSummaryCount,
    refresh,
  } as const;
}
