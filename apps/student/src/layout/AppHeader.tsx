import { useDeviceStore } from "@/store/deviceStore";
import { useExamStore } from "@/store/examStore";
import { useEffect } from "react";
import linkIcon from "@/assets/icons/link.png";
import linkSuccessIcon from "@/assets/icons/link_success.png";
import linkFailIcon from "@/assets/icons/link_fail.png";

export default function AppHeader() {
  const currentSession = useExamStore((s) => s.currentSession);
  const ip = useDeviceStore((s) => s.ip);
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

  const teacherStatusIconMap: Record<string, string> = {
    connected: linkSuccessIcon,
    disconnected: linkFailIcon,
    connecting: linkIcon,
    unknown: linkFailIcon,
  };

  const imgIcon = teacherStatusIconMap[teacherStatus] ?? linkFailIcon;
  const statusText = teacherStatusTextMap[teacherStatus] ?? teacherStatusTextMap.unknown;
  const isConnecting = teacherStatus === "connecting";
  const isConnected = teacherStatus === "connected";

  const examText = isConnected && currentSession
    ? `${currentSession.examTitle ?? "未命名考试"}`
    : "未加入考试";
  const studentText = isConnected && currentSession
    ? `${currentSession.studentNo} ${currentSession.studentName}`
    : "未分配学生";


  return (
    <header className="border-slate-200 bg-white px-4 py-3">
      <div className="flex w-full items-center justify-between ">
        <div className="text-base text-slate-900 space-x-4">
          <span>
            {examText}
          </span>
          <span>{'教师端 IP: '  + (teacherEndpoint ? teacherEndpoint.slice(5,15) : "未配置")}</span>

          <img
              src={imgIcon}
              alt={`连接状态:${statusText}`}
              className={`h-5 w-5 ${isConnecting ? "animate-pulse" : ""}`}
            />
          <span>
          </span>
        </div>

        <div className="space-x-4">
          <span>{ "学生: " + studentText }</span>
          <span>{ip ? `设备 IP: ${ip}` : "设备 IP：未知"}</span>
        </div>
      </div>
    </header>
  );
}
