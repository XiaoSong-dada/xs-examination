import type { IStudentCreate, IStudentEditor, StudentListItem } from "@/types/main";
import { invoke } from "@tauri-apps/api/core";

export async function getStudentList(): Promise<StudentListItem[]> {
  return invoke<StudentListItem[]>("get_students");
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
