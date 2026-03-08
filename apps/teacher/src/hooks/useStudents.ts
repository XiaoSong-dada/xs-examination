import { useCallback, useEffect, useMemo, useState } from "react";
import type {
  IStudentCreate,
  IStudentEditor,
  StudentListItem,
  UseStudentListResult,
} from "@/types/main";
import {
  bulkCreateStudents as bulkCreate,
  createStudent as create,
  deleteStudent as remove,
  getStudentList,
  updateStudent as update,
} from "@/services/studentService";
import { deepClone } from "@/utils/utils";

export function useStudentList(): UseStudentListResult {
  const [loading, setLoading] = useState<boolean>(false);
  const [allStudents, setAllStudents] = useState<StudentListItem[]>([]);
  const [inputKeyword, setInputKeyword] = useState("");
  const [appliedKeyword, setAppliedKeyword] = useState("");
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);

  const refresh = useCallback(async (): Promise<void> => {
    setLoading(true);
    try {
      const result = await getStudentList();
      setAllStudents(result);
    } catch (error) {
      console.error("[useStudentList] 获取学生列表失败", error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const search = useCallback(() => {
    setAppliedKeyword(inputKeyword);
    setPage(1);
  }, [inputKeyword]);

  const reset = useCallback(() => {
    setInputKeyword("");
    setAppliedKeyword("");
    setPage(1);
  }, []);

  const filteredStudents = useMemo(() => {
    const keyword = appliedKeyword.trim().toLowerCase();
    if (!keyword) {
      return allStudents;
    }

    return allStudents.filter((item) => {
      return (
        item.name.toLowerCase().includes(keyword) ||
        item.student_no.toLowerCase().includes(keyword)
      );
    });
  }, [allStudents, appliedKeyword]);

  const total = filteredStudents.length;

  const dataSource = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredStudents.slice(start, start + pageSize);
  }, [filteredStudents, page, pageSize]);

  useEffect(() => {
    const maxPage = Math.max(1, Math.ceil(total / pageSize));
    if (page > maxPage) {
      setPage(maxPage);
    }
  }, [page, pageSize, total]);

  return {
    loading,
    inputKeyword,
    appliedKeyword,
    setInputKeyword,
    search,
    reset,
    page,
    pageSize,
    setPage,
    setPageSize,
    total,
    dataSource,
    refresh,
  };
}

export const useCreateStudent = () => {
  const createStudent = useCallback(async (data: IStudentCreate) => {
    try {
      await create(data);
      return true;
    } catch (e) {
      console.error("创建学生失败", e);
      return false;
    }
  }, []);

  return { createStudent };
};

export const useUpdateStudent = () => {
  const updateStudent = useCallback(async (data: IStudentEditor) => {
    try {
      await update(data);
      return true;
    } catch (e) {
      console.error("更新学生失败", e);
      return false;
    }
  }, []);

  return { updateStudent };
};

export const useDeleteStudent = () => {
  const deleteStudent = useCallback(async (id: string) => {
    try {
      await remove(id);
      return true;
    } catch (e) {
      console.error("删除学生失败", e);
      return false;
    }
  }, []);

  return { deleteStudent };
};

export const useBulkCreateStudents = () => {
  const bulkCreateStudents = useCallback(async (students: IStudentCreate[]) => {
    try {
      const result = await bulkCreate(students);
      return result;
    } catch (e) {
      console.error("批量导入学生失败", e);
      return null;
    }
  }, []);

  return { bulkCreateStudents };
};

const defaultCreateStudentData: IStudentEditor = {
  id: "",
  student_no: "",
  name: "",
  created_at: undefined,
  updated_at: undefined,
};

export function useStudentModal() {
  const [visible, setVisible] = useState(false);
  const [modalTitle, setModalTitle] = useState("新增学生");
  const [formData, setFormData] = useState<IStudentEditor | null>(null);

  const openCreate = useCallback(() => {
    setModalTitle("新增学生");
    setFormData(deepClone(defaultCreateStudentData));
    setVisible(true);
  }, []);

  const openEdit = useCallback((data: IStudentEditor) => {
    setModalTitle("编辑学生");
    setFormData(data);
    setVisible(true);
  }, []);

  const close = useCallback(() => {
    setVisible(false);
  }, []);

  return {
    modalTitle,
    visible,
    formData,
    setFormData,
    openCreate,
    openEdit,
    close,
  } as const;
}
