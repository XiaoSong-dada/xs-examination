import { useDeviceStore } from "@/store/deviceStore";
import { useExamStore } from "@/store/examStore";
import { Button } from "antd";
import { useEffect } from "react";
export default function AppHeader() {
  const currentExam = useExamStore((s) => s.currentExam);
  const ip = useDeviceStore((s) => s.ip);
  const assigned = useDeviceStore((s) => s.assignedStudent);
  const teacherEndpoint = useDeviceStore((s) => s.teacherMasterEndpoint);
  const teacherStatus = useDeviceStore((s) => s.teacherConnectionStatus);
  const initTeacherInfo = useDeviceStore((s) => s.initTeacherInfo);

  useEffect(() => {
    void initTeacherInfo();
  }, [initTeacherInfo]);

  const teacherStatusTextMap: Record<string, string> = {
    connected: "已连接",
    disconnected: "未连接",
    connecting: "连接中",
    unknown: "未知",
  };

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
          <span>{`主教师: ${teacherEndpoint ?? "未配置"}`}</span>
          <span>{`教师端状态: ${teacherStatusTextMap[teacherStatus] ?? "未知"}`}</span>
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
