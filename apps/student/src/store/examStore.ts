import { create } from "zustand";
import type { ExamStore, RuntimeQuestion } from "@/types/main";
import type { ExamQuestionOption } from "@/types/exam";
import { getCurrentExamBundle } from "@/services/examRuntimeService";

function parseOptions(raw?: unknown): ExamQuestionOption[] {
    if (raw == null)return [];
    if (Array.isArray(raw)) return raw as ExamQuestionOption[];
    try {
        const parsed = JSON.parse(String(raw)) as ExamQuestionOption[];
        if (Array.isArray(parsed))return parsed;
    } catch (_err) {
        console.error("Failed to parse options:", _err);
    }

    const fallback = String(raw).trim();
    return fallback ? [{ key: "1", text: fallback }] : [];
}

function parseQuestions(payload?: string): RuntimeQuestion[] {
    if (!payload) {
        return [];
    }

    try {
        const parsed = JSON.parse(payload) as Array<Record<string, unknown>>;
        if (!Array.isArray(parsed)) {
            return [];
        }

        return parsed.map((item) => ({
            id: String(item.id ?? ""),
            seq: Number(item.seq ?? 0),
            type: String(item.type ?? "single"),
            content: String(item.content ?? ""),
            options: parseOptions(item.options),
            score: Number(item.score ?? 0),
            explanation: typeof item.explanation === "string" ? item.explanation : undefined,
            images: [],
        }));
    } catch (_err) {
        return [];
    }
}

export const useExamStore = create<ExamStore>((set) => ({
    currentExam: null,
    currentSession: null,
    currentSnapshot: null,
    questions: [],
    loading: false,
    setCurrentExam: (exam) => set({ currentExam: exam }),
    refreshCurrentExam: async () => {
        set({ loading: true });
        try {
            const bundle = await getCurrentExamBundle();
            const session = bundle.session ?? null;
            const snapshot = bundle.snapshot ?? null;
            
            set({
                currentSession: session,
                currentSnapshot: snapshot,
                currentExam: session
                    ? {
                            id: session.examId,
                            title: session.examTitle,
                            status: session.status,
                        }
                    : null,
                questions: parseQuestions(snapshot?.questionsPayload),
            });
        } finally {
            set({ loading: false });
        }
    },
}));

export default useExamStore;
