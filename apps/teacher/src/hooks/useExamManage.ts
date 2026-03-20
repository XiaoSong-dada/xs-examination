import { useCallback, useEffect, useMemo, useState } from "react";

import { useAllExamList } from "@/hooks/useExam";
import { getExamById, updateExam } from "@/services/examService";
import {
  distributeExamPapersByExamId,
  startExamByExamId,
} from "@/services/studentService";
import type {
  DeviceConnectionStatus,
  StudentDeviceConnectionStatusItem,
} from "@/types/main";
import { useDeviceAssign } from "./useDeviceAssign";

export interface ExamManageTableItem {
  id: string;
  name: string;
  deviceIp: string;
  deviceStatus: DeviceConnectionStatus;
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
  } = useDeviceAssign();

  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [students, setStudents] = useState<StudentDeviceConnectionStatusItem[]>([]);
  const [tableLoading, setTableLoading] = useState(false);
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

  const loadStudents = useCallback(async (examId?: string) => {
    if (!examId) {
      setStudents([]);
      return;
    }

    setTableLoading(true);
    try {
      const studentList = await getAssignStudentByExamId(examId);
      setStudents(studentList ?? []);
    } finally {
      setTableLoading(false);
    }
  }, [getAssignStudentByExamId]);

  useEffect(() => {
    void Promise.all([
      loadStudents(selectedExamId),
      loadExamStatus(selectedExamId),
    ]);
  }, [loadExamStatus, loadStudents, selectedExamId]);

  useEffect(() => {
    if (!selectedExamId) {
      return;
    }

    const timer = window.setInterval(() => {
      void loadStudents(selectedExamId);
    }, 5000);

    return () => {
      window.clearInterval(timer);
    };
  }, [loadStudents, selectedExamId]);

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
      await Promise.all([loadExamStatus(selectedExamId), loadStudents(selectedExamId)]);
      return result;
    } finally {
      setDistributing(false);
    }
  }, [loadExamStatus, loadStudents, refreshExamList, selectedExamId]);

  const startExam = useCallback(async () => {
    if (!selectedExamId) {
      return null;
    }

    setStarting(true);
    try {
      const result = await startExamByExamId(selectedExamId);
      await refreshExamList();
      await Promise.all([loadExamStatus(selectedExamId), loadStudents(selectedExamId)]);
      return result;
    } finally {
      setStarting(false);
    }
  }, [loadExamStatus, loadStudents, refreshExamList, selectedExamId]);

  const examOptions = useMemo(
    () => exams.map((exam) => ({ label: exam.title, value: exam.id })),
    [exams],
  );

  const tableData = useMemo<ExamManageTableItem[]>(
    () =>
      students.map((student: StudentDeviceConnectionStatusItem) => ({
        id: student.student_id,
        name: student.student_name,
        deviceIp: student.ip_addr ?? "-",
        deviceStatus: student.connection_status,
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
    tableLoading,
    distributePapers,
    startExam,
    distributing,
    starting,
  } as const;
}
