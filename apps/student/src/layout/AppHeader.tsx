import { useDeviceStore } from "@/store/deviceStore";
import { useExamStore } from "@/store/examStore";
import { useEffect, useState } from "react";
import linkIcon from "@/assets/icons/link.png";
import linkSuccessIcon from "@/assets/icons/link_success.png";
import linkFailIcon from "@/assets/icons/link_fail.png";

function endpointToHost(endpoint: string | null): string {
  if (!endpoint) {
    return "未配置";
  }

  try {
    return new URL(endpoint).hostname || endpoint;
  } catch (_err) {
    return endpoint;
  }
}

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
  const [blinkVisible, setBlinkVisible] = useState(true);

  useEffect(() => {
    if (!isConnecting) {
      setBlinkVisible(true);
      return;
    }

    const timer = window.setInterval(() => {
      setBlinkVisible((prev) => !prev);
    }, 800);

    return () => {
      window.clearInterval(timer);
    };
  }, [isConnecting]);

  const examText = currentSession
    ? `${currentSession.examTitle ?? "未命名考试"}`
    : "未加入考试";
  const studentText = currentSession
    ? `${currentSession.studentNo} ${currentSession.studentName}`
    : "未分配学生";


  return (
    <header className="border-slate-200 bg-white px-4 py-3">
      <div className="flex w-full items-center justify-between ">
        <div className="text-base text-slate-900 space-x-4">
          <span>
            {examText}
          </span>
          <span>{`教师端 IP: ${endpointToHost(teacherEndpoint)}`}</span>

          <img
              src={imgIcon}
              alt={`连接状态:${statusText}`}
              className={`h-5 w-5 -mt-1 transition-opacity ${isConnecting && !blinkVisible ? "opacity-0" : "opacity-100"}`}
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
