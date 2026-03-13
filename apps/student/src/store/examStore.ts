import { create } from "zustand";
import { ExamStore } from "@/types/main";


export const useExamStore = create<ExamStore>((set) => ({
    currentExam: null,
    setCurrentExam: (exam) => set({ currentExam: exam }),
}));

export default useExamStore;
