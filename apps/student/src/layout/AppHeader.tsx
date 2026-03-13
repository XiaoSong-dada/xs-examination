import { useDeviceStore } from "@/store/deviceStore";
import { useExamStore } from "@/store/examStore";

export default function AppHeader() {
  const currentExam = useExamStore((s) => s.currentExam);
  const ip = useDeviceStore((s) => s.ip);
  const assigned = useDeviceStore((s) => s.assignedStudent);

  return (
    <header className="border-b border-slate-200 bg-white px-4 py-3">
      <div className="mx-auto flex w-full max-w-7xl items-center justify-between">
        <div className="text-base font-semibold text-slate-900">
          {currentExam
            ? `${currentExam.title ?? "未命名考试"} · ${currentExam.status ?? "未知状态"}`
            : "未加入考试"}
        </div>

        <div className="text-right text-sm text-slate-600">
          <div>{ip ? `设备 IP: ${ip}` : "设备 IP：未知"}</div>
          <div>{assigned ? `${assigned.studentNo} ${assigned.name}` : "未分配学生"}</div>
        </div>
      </div>
    </header>
  );
}
