import { useDeviceStore } from "@/store/deviceStore";
import { useExamStore } from "@/store/examStore";
import { Button } from "antd";
export default function AppHeader() {
  const currentExam = useExamStore((s) => s.currentExam);
  const ip = useDeviceStore((s) => s.ip);
  const assigned = useDeviceStore((s) => s.assignedStudent);

  return (
    <header className="border-slate-200 bg-white px-4 py-3">
      <div className="flex w-full items-center justify-between ">
        <div className="text-base text-slate-900 space-x-4">
          <span>
            {currentExam
              ? `${currentExam.title ?? "未命名考试"} · ${currentExam.status ?? "未知状态"}`
              : "未加入考试"}
          </span>
          <span>{ip ? `设备 IP: ${ip}` : "设备 IP：未知"}</span>
          <span>
            {assigned ? `${assigned.studentNo} ${assigned.name}` : "未分配学生"}
          </span>
        </div>

        <div>
          <Button type="primary">查看考试详情</Button>
        </div>
      </div>
    </header>
  );
}
