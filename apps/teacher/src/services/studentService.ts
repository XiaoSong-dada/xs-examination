import type {
  DistributeExamPapersResult,
  EndExamResult,
  IStudentCreate,
  IStudentEditor,
  PushTeacherEndpointsResult,
  StartExamResult,
  StudentDeviceConnectionStatusItem,
  StudentDeviceAssignItem,
  StudentDeviceAssignPayloadItem,
  StudentScoreSummaryItem,
  StudentListItem,
} from "@/types/main";
import { invoke } from "@tauri-apps/api/core";

export async function getStudentList(): Promise<StudentListItem[]> {
  return invoke<StudentListItem[]>("get_students");
}

export async function getStudentListByExamId(examId: string): Promise<StudentListItem[]> {
  return invoke<StudentListItem[]>("get_students_by_exam_id", {
    payload: { exam_id: examId },
  });
}

export async function importStudentsByExamId(
  examId: string,
  studentIds: string[],
): Promise<StudentListItem[]> {
  return invoke<StudentListItem[]>("import_students_by_exam_id", {
    payload: {
      exam_id: examId,
      student_ids: studentIds,
    },
  });
}

export async function getStudentDeviceAssignmentsByExamId(
  examId: string,
): Promise<StudentDeviceAssignItem[]> {
  return invoke<StudentDeviceAssignItem[]>("get_student_device_assignments_by_exam_id", {
    payload: { exam_id: examId },
  });
}

export async function assignDevicesToStudentExams(
  examId: string,
  assignments: StudentDeviceAssignPayloadItem[],
): Promise<StudentDeviceAssignItem[]> {
  return invoke<StudentDeviceAssignItem[]>("assign_devices_to_student_exams", {
    payload: {
      exam_id: examId,
      assignments,
    },
  });
}

export async function connectStudentDevicesByExamId(
  examId: string,
): Promise<PushTeacherEndpointsResult> {
  return invoke<PushTeacherEndpointsResult>("connect_student_devices_by_exam_id", {
    payload: { exam_id: examId },
  });
}

export async function getStudentDeviceConnectionStatusByExamId(
  examId: string,
): Promise<StudentDeviceConnectionStatusItem[]> {
  return invoke<StudentDeviceConnectionStatusItem[]>(
    "get_student_device_connection_status_by_exam_id",
    {
      payload: { exam_id: examId },
    },
  );
}

/**
 * 查询指定考试的成绩汇总结果。
 * @param examId 考试 ID。
 * @returns 返回已落库的学生总分列表。
 */
export async function getStudentScoreSummaryByExamId(
  examId: string,
): Promise<StudentScoreSummaryItem[]> {
  return invoke<StudentScoreSummaryItem[]>("get_student_score_summary_by_exam_id", {
    payload: { exam_id: examId },
  });
}

/**
 * 触发指定考试的成绩统计并覆盖写入数据库。
 * @param examId 考试 ID。
 * @returns 返回重算后的学生总分列表。
 */
export async function calculateStudentScoreSummaryByExamId(
  examId: string,
): Promise<StudentScoreSummaryItem[]> {
  return invoke<StudentScoreSummaryItem[]>("calculate_student_score_summary_by_exam_id", {
    payload: { exam_id: examId },
  });
}

/**
 * 将成绩报告二进制写入本机文件并返回保存路径。
 * @param fileName 导出文件名。
 * @param bytes xlsx 二进制字节数组。
 * @returns 返回后端实际写入的绝对路径。
 */
export async function saveScoreReportFile(
  fileName: string,
  bytes: number[],
): Promise<{ path: string }> {
  return invoke<{ path: string }>("save_score_report_file", {
    payload: {
      file_name: fileName,
      bytes,
    },
  });
}

export async function distributeExamPapersByExamId(
  examId: string,
): Promise<DistributeExamPapersResult> {
  return invoke<DistributeExamPapersResult>("distribute_exam_papers_by_exam_id", {
    payload: { exam_id: examId },
  });
}

export async function startExamByExamId(
  examId: string,
): Promise<StartExamResult> {
  return invoke<StartExamResult>("start_exam_by_exam_id", {
    payload: { exam_id: examId },
  });
}

/**
 * 调用教师端 `end_exam_by_exam_id` 命令，触发在线学生最终同步并结束考试。
 * @param examId 考试 ID。
 * @returns 结束考试下发与 ACK 聚合结果。
 */
export async function endExamByExamId(
  examId: string,
): Promise<EndExamResult> {
  return invoke<EndExamResult>("end_exam_by_exam_id", {
    payload: { exam_id: examId },
  });
}

export async function getStudentById(id: string): Promise<StudentListItem> {
  return invoke<StudentListItem>("get_student_by_id", { payload: { id } });
}

export async function createStudent(data: IStudentCreate) {
  return invoke("create_student", { payload: data });
}

export async function updateStudent(data: IStudentEditor) {
  return invoke("update_student", { payload: data });
}

export async function deleteStudent(id: string) {
  return invoke("delete_student", { payload: { id } });
}

export async function bulkCreateStudents(data: IStudentCreate[]) {
  return invoke<StudentListItem[]>("bulk_create_students", {
    payload: { students: data },
  });
}
