import { useCallback, useEffect, useMemo, useState } from "react";

import { useAllExamList } from "@/hooks/useExam";
import { getExamById, updateExam } from "@/services/examService";
import {
  distributeExamPapersByExamId,
  startExamByExamId,
} from "@/services/studentService";
import type { StudentDeviceAssignItem } from "@/types/main";
import { useDeviceAssign } from "./useDeviceAssign";

export interface ExamManageTableItem {
  id: string;
  name: string;
  deviceIp: string;
  linkStatus: string;
  status: string;
}

const examStatusLabelMap: Record<string, string> = {
  draft: "草稿",
  published: "已发卷",
  active: "考试中",
  finished: "已结束",
  archived: "已归档",
};

/**
 * 考试管理页面 Hook：负责考试选择、状态变更与学生表格数据。
 */
export function useExamManage() {
  const { exams, loading: examLoading, refresh: refreshExamList } = useAllExamList();
  const {
    getAssignStudentByExamId,
    loading: studentLoading,
  } = useDeviceAssign();

  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [students, setStudents] = useState<StudentDeviceAssignItem[]>([]);
  const [currentExamStatus, setCurrentExamStatus] = useState<string>("draft");
  const [currentExamEndTime, setCurrentExamEndTime] = useState<number | undefined>();
  const [distributing, setDistributing] = useState(false);
  const [starting, setStarting] = useState(false);

  useEffect(() => {
    if (!selectedExamId && exams.length > 0) {
      setSelectedExamId(exams[0].id);
    }
  }, [exams, selectedExamId]);

  const loadExamStatus = useCallback(async (examId?: string) => {
    if (!examId) {
      setCurrentExamStatus("draft");
      setCurrentExamEndTime(undefined);
      return;
    }

    try {
      const detail = await getExamById(examId);
      setCurrentExamStatus(detail.status ?? "draft");
      setCurrentExamEndTime(
        typeof detail.end_time === "number" ? detail.end_time : undefined,
      );
    } catch (error) {
      console.error("[useExamManage] 获取考试详情失败", error);
      setCurrentExamStatus("draft");
      setCurrentExamEndTime(undefined);
    }
  }, []);

  useEffect(() => {
    const loadData = async () => {
      const student = await getAssignStudentByExamId(selectedExamId ?? "");
      setStudents(student ?? []);
      await loadExamStatus(selectedExamId);
    };

    void loadData();
  }, [getAssignStudentByExamId, loadExamStatus, selectedExamId]);

  const updateExamStatus = useCallback(
    async (status: string) => {
      if (!selectedExamId) {
        return false;
      }

      try {
        const detail = await getExamById(selectedExamId);
        await updateExam({
          id: detail.id,
          title: detail.title,
          description: detail.description,
          start_time: detail.start_time,
          end_time: detail.end_time,
          pass_score: detail.pass_score,
          status,
          shuffle_questions: detail.shuffle_questions,
          shuffle_options: detail.shuffle_options,
        });

        setCurrentExamStatus(status);
        await refreshExamList();
        return true;
      } catch (error) {
        console.error("[useExamManage] 更新考试状态失败", error);
        return false;
      }
    },
    [refreshExamList, selectedExamId],
  );

  useEffect(() => {
    if (!selectedExamId || !currentExamEndTime || currentExamStatus !== "active") {
      return;
    }

    const msLeft = currentExamEndTime - Date.now();
    if (msLeft <= 0) {
      void updateExamStatus("finished");
      return;
    }

    const timer = window.setTimeout(() => {
      void updateExamStatus("finished");
    }, msLeft);

    return () => {
      window.clearTimeout(timer);
    };
  }, [currentExamEndTime, currentExamStatus, selectedExamId, updateExamStatus]);

  const distributePapers = useCallback(async () => {
    if (!selectedExamId) {
      return null;
    }

    setDistributing(true);
    try {
      const result = await distributeExamPapersByExamId(selectedExamId);
      await refreshExamList();
      await loadExamStatus(selectedExamId);
      return result;
    } finally {
      setDistributing(false);
    }
  }, [loadExamStatus, refreshExamList, selectedExamId]);

  const startExam = useCallback(async () => {
    if (!selectedExamId) {
      return null;
    }

    setStarting(true);
    try {
      const result = await startExamByExamId(selectedExamId);
      await refreshExamList();
      await loadExamStatus(selectedExamId);
      return result;
    } finally {
      setStarting(false);
    }
  }, [loadExamStatus, refreshExamList, selectedExamId]);

  const examOptions = useMemo(
    () => exams.map((exam) => ({ label: exam.title, value: exam.id })),
    [exams],
  );

  const tableData = useMemo<ExamManageTableItem[]>(
    () =>
      students.map((student: StudentDeviceAssignItem) => ({
        id: student.student_id,
        name: student.student_name,
        deviceIp: student.ip_addr ?? "-",
        linkStatus: student.ip_addr ? "已连接" : "未连接",
        status: student.ip_addr ? "已分配" : "待考",
      })),
    [students],
  );

  return {
    selectedExamId,
    setSelectedExamId,
    examOptions,
    examLoading,
    currentExamStatus,
    currentExamStatusLabel: examStatusLabelMap[currentExamStatus] ?? currentExamStatus,
    tableData,
    tableLoading: studentLoading,
    distributePapers,
    startExam,
    distributing,
    starting,
  } as const;
}
