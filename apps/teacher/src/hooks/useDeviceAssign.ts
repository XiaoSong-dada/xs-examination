import { useCallback, useEffect, useMemo, useState } from "react";
import type {
  DeviceAssignRow,
  DeviceListItem,
  ExamOption,
  PushTeacherEndpointsResult,
  StudentDeviceConnectionStatusItem,
  StudentDeviceAssignPayloadItem,
} from "@/types/main";
import { getDeviceList } from "@/services/deviceService";
import { getExamList } from "@/services/examService";
import {
  assignDevicesToStudentExams,
  connectStudentDevicesByExamId,
  getStudentDeviceConnectionStatusByExamId,
} from "@/services/studentService";


function shuffleArray<T>(items: T[]): T[] {
  const result = [...items];
  for (let i = result.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1));
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

function buildRows(
  assignments: StudentDeviceConnectionStatusItem[],
): DeviceAssignRow[] {
  return assignments.map((item) => ({
    id: item.student_exam_id,
    student_exam_id: item.student_exam_id,
    student_id: item.student_id,
    student_no: item.student_no,
    student_name: item.student_name,
    ip_addr: item.ip_addr,
    device_name: item.device_name,
    assigned: Boolean(item.ip_addr),
    connection_status: item.connection_status,
    last_heartbeat_at: item.last_heartbeat_at,
    has_heartbeat_seen: item.has_heartbeat_seen,
  }));
}

export function useDeviceAssign() {
  const [loading, setLoading] = useState(false);
  const [assigning, setAssigning] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [examOptions, setExamOptions] = useState<ExamOption[]>([]);
  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [allDevices, setAllDevices] = useState<DeviceListItem[]>([]);
  const [allAssignments, setAllAssignments] = useState<StudentDeviceConnectionStatusItem[]>([]);

  const loadExams = useCallback(async () => {
    const exams = await getExamList();
    setExamOptions(
      exams.map((exam) => ({
        label: exam.title,
        value: exam.id,
      })),
    );
  }, []);

  const loadDevices = useCallback(async () => {
    const devices = await getDeviceList();
    setAllDevices(devices);
  }, []);

  const loadAssignments = useCallback(async (examId: string) => {
    if (!examId) {
      setAllAssignments([]);
      return;
    }
    const assignments = await getStudentDeviceConnectionStatusByExamId(examId);
    setAllAssignments(assignments);
  }, []);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      await Promise.all([loadExams(), loadDevices()]);
    } finally {
      setLoading(false);
    }
  }, [loadDevices, loadExams]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!selectedExamId) {
      setAllAssignments([]);
      return;
    }

    setLoading(true);
    void loadAssignments(selectedExamId)
      .finally(() => {
        setLoading(false);
      });
  }, [loadAssignments, selectedExamId]);

  useEffect(() => {
    if (!selectedExamId) {
      return;
    }

    const timer = window.setInterval(() => {
      void loadAssignments(selectedExamId);
    }, 5000);

    return () => {
      window.clearInterval(timer);
    };
  }, [loadAssignments, selectedExamId]);

  const randomAssign = useCallback(async () => {
    if (!selectedExamId || allAssignments.length === 0 || allDevices.length === 0) {
      return false;
    }

    setAssigning(true);
    try {
      const shuffledStudents = shuffleArray(allAssignments);
      const shuffledDevices = shuffleArray(allDevices);
      const assignCount = Math.min(shuffledStudents.length, shuffledDevices.length);
      const nextIpMap = new Map<string, string | undefined>();

      for (const item of allAssignments) {
        nextIpMap.set(item.student_exam_id, undefined);
      }

      for (let i = 0; i < assignCount; i += 1) {
        nextIpMap.set(shuffledStudents[i].student_exam_id, shuffledDevices[i].ip);
      }

      const payload: StudentDeviceAssignPayloadItem[] = allAssignments.map((item) => ({
        student_exam_id: item.student_exam_id,
        ip_addr: nextIpMap.get(item.student_exam_id),
      }));

      await assignDevicesToStudentExams(selectedExamId, payload);
      await loadAssignments(selectedExamId);
      return true;
    } finally {
      setAssigning(false);
    }
  }, [allAssignments, allDevices, loadAssignments, selectedExamId]);

  const clearAssign = useCallback(async () => {
    if (!selectedExamId || allAssignments.length === 0) {
      return;
    }

    setAssigning(true);
    try {
      const payload: StudentDeviceAssignPayloadItem[] = allAssignments.map((item) => ({
        student_exam_id: item.student_exam_id,
        ip_addr: undefined,
      }));
      await assignDevicesToStudentExams(selectedExamId, payload);
      await loadAssignments(selectedExamId);
    } finally {
      setAssigning(false);
    }
  }, [allAssignments, loadAssignments, selectedExamId]);

  const connectDevices = useCallback(async (): Promise<PushTeacherEndpointsResult | null> => {
    if (!selectedExamId) {
      return null;
    }

    setConnecting(true);
    try {
      const result = await connectStudentDevicesByExamId(selectedExamId);
      await loadAssignments(selectedExamId);
      return result;
    } finally {
      setConnecting(false);
    }
  }, [loadAssignments, selectedExamId]);

  const tableData = useMemo(
    () => buildRows(allAssignments),
    [allAssignments],
  );

  const getAssignStudentByExamId = useCallback(async (examId: string) => {
    if (!examId) {
      return undefined;
    }
    const student = await getStudentDeviceConnectionStatusByExamId(examId);
    return student;
  }, []);

  return {
    loading,
    assigning,
    connecting,
    examOptions,
    selectedExamId,
    setSelectedExamId,
    tableData,
    randomAssign,
    clearAssign,
    connectDevices,
    studentCount: allAssignments.length,
    deviceCount: allDevices.length,
    assignedCount: tableData.filter((item) => item.assigned).length,
    getAssignStudentByExamId,
    refresh,
  } as const;
}
