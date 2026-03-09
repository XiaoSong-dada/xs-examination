import type { IStudentCreate, IStudentEditor, StudentListItem } from "@/types/main";
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
