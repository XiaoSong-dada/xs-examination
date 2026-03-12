import { useCallback, useEffect, useMemo, useState } from "react";
import type { DeviceListItem, StudentListItem } from "@/types/main";
import { getDeviceList } from "@/services/deviceService";
import { getExamList } from "@/services/examService";
import { getStudentListByExamId } from "@/services/studentService";

export interface DeviceAssignRow {
  id: string;
  ip: string;
  name: string;
  student_id?: string;
  student_no?: string;
  student_name?: string;
  assigned: boolean;
}

export interface ExamOption {
  label: string;
  value: string;
}

function shuffleArray<T>(items: T[]): T[] {
  const result = [...items];
  for (let i = result.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1));
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

function buildRows(
  devices: DeviceListItem[],
  assignedMap: Map<string, StudentListItem>,
): DeviceAssignRow[] {
  return devices.map((device) => {
    const student = assignedMap.get(device.id);
    return {
      id: device.id,
      ip: device.ip,
      name: device.name,
      student_id: student?.id,
      student_no: student?.student_no,
      student_name: student?.name,
      assigned: Boolean(student),
    };
  });
}

export function useDeviceAssign() {
  const [loading, setLoading] = useState(false);
  const [assigning, setAssigning] = useState(false);
  const [examOptions, setExamOptions] = useState<ExamOption[]>([]);
  const [selectedExamId, setSelectedExamId] = useState<string>();
  const [allDevices, setAllDevices] = useState<DeviceListItem[]>([]);
  const [allStudents, setAllStudents] = useState<StudentListItem[]>([]);
  const [assignedMap, setAssignedMap] = useState<Map<string, StudentListItem>>(new Map());

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
      setAllStudents([]);
      setAssignedMap(new Map());
      return;
    }

    setLoading(true);
    void getStudentListByExamId(selectedExamId)
      .then((students) => {
        setAllStudents(students);
        setAssignedMap(new Map());
      })
      .finally(() => {
        setLoading(false);
      });
  }, [selectedExamId]);

  const randomAssign = useCallback(() => {
    if (!selectedExamId || allStudents.length === 0 || allDevices.length === 0) {
      return false;
    }

    setAssigning(true);
    const shuffledStudents = shuffleArray(allStudents);
    const nextAssignedMap = new Map<string, StudentListItem>();
    const assignCount = Math.min(allDevices.length, shuffledStudents.length);

    for (let i = 0; i < assignCount; i += 1) {
      nextAssignedMap.set(allDevices[i].id, shuffledStudents[i]);
    }

    setAssignedMap(nextAssignedMap);
    setAssigning(false);
    return true;
  }, [allDevices, allStudents, selectedExamId]);

  const clearAssign = useCallback(() => {
    setAssignedMap(new Map());
  }, []);

  const tableData = useMemo(
    () => buildRows(allDevices, assignedMap),
    [allDevices, assignedMap],
  );

  return {
    loading,
    assigning,
    examOptions,
    selectedExamId,
    setSelectedExamId,
    tableData,
    randomAssign,
    clearAssign,
    studentCount: allStudents.length,
    deviceCount: allDevices.length,
    assignedCount: assignedMap.size,
    refresh,
  } as const;
}
