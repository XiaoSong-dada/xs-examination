import AppLayout from "@/layout/AppLayout";
import ExamPage from "@/pages/Exam";
import { useEffect } from "react";
import { useExamStore } from "@/store/examStore";

function WaitingView({ text }: { text: string }) {
  return (
    <main className="h-full border border-slate-200 bg-white p-6 shadow-sm flex items-center justify-center">
      <div className="text-center space-y-2">
        <h1 className="text-xl font-semibold text-slate-900">等待考试开始</h1>
        <p className="text-slate-600">{text}</p>
      </div>
    </main>
  );
}

function App() {
  const currentSession = useExamStore((s) => s.currentSession);
  const currentSnapshot = useExamStore((s) => s.currentSnapshot);
  const loading = useExamStore((s) => s.loading);
  const refreshCurrentExam = useExamStore((s) => s.refreshCurrentExam);

  useEffect(() => {
    void refreshCurrentExam();
    const timer = window.setInterval(() => {
      void refreshCurrentExam();
    }, 3000);

    return () => {
      window.clearInterval(timer);
    };
  }, [refreshCurrentExam]);

  const shouldShowExam =
    currentSession?.status === "active" && Boolean(currentSnapshot);

  return (
    <AppLayout>
      {loading && !currentSession ? (
        <WaitingView text="正在加载考试状态..." />
      ) : shouldShowExam ? (
        <ExamPage />
      ) : (
        <WaitingView
          text={
            currentSession
              ? "试卷已下发，请等待教师开始考试指令"
              : "尚未收到教师下发的考试试卷"
          }
        />
      )}
    </AppLayout>
  );
}

export default App;
