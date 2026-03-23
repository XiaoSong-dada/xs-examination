import AppLayout from "@/layout/AppLayout";
import ExamPage from "@/pages/Exam";
import { useEffect } from "react";
import { useExamStore } from "@/store/examStore";
import { useDeviceStore } from "@/store/deviceStore";
import { onExamStatusChanged } from "@/services/examRuntimeService";

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
  const teacherStatus = useDeviceStore((s) => s.teacherConnectionStatus);

  useEffect(() => {
    void refreshCurrentExam();
    const timer = window.setInterval(() => {
      void refreshCurrentExam();
    }, 3000);

    return () => {
      window.clearInterval(timer);
    };
  }, [refreshCurrentExam]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    void onExamStatusChanged(() => {
      void refreshCurrentExam();
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [refreshCurrentExam]);

  const shouldShowExam =
    currentSession?.status === "active" && Boolean(currentSnapshot);

  let waitingText = "尚未收到教师下发的考试试卷";
  if (currentSession && !currentSnapshot) {
    waitingText = teacherStatus === "connected"
      ? "考生设备已连接，等待教师下发试卷"
      : "已写入考生会话，正在连接教师端";
  } else if (currentSession && currentSnapshot) {
    waitingText = "试卷已下发，请等待教师开始考试指令";
  }

  return (
    <AppLayout>
      {loading && !currentSession ? (
        <WaitingView text="正在加载考试状态..." />
      ) : shouldShowExam ? (
        <ExamPage />
      ) : (
        <WaitingView
          text={waitingText}
        />
      )}
    </AppLayout>
  );
}

export default App;
