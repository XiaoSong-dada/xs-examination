import { create } from "zustand";
import { DeviceStore, TeacherConnectionStatus } from "@/types/main";
import {
  getTeacherRuntimeStatus,
  onTeacherEndpointApplied,
  onWsConnected,
  onWsDisconnected,
} from "@/services/teacherEndpointService";

let hasSubscribedTeacherEvents = false;

export const useDeviceStore = create<DeviceStore>((set, get) => ({
  ip: null,
  assignedStudent: null,
  teacherMasterEndpoint: null,
  teacherConnectionStatus: "unknown",

  setIp: (ip) => set({ ip }),
  setAssignedStudent: (assignedStudent) => set({ assignedStudent }),

  setTeacherMasterEndpoint: (ep) => set({ teacherMasterEndpoint: ep }),
  setTeacherConnectionStatus: (s: TeacherConnectionStatus) => set({ teacherConnectionStatus: s }),

  async initTeacherInfo() {
    try {
      const runtime = await getTeacherRuntimeStatus();
      get().setTeacherMasterEndpoint(runtime.endpoint ?? null);
      get().setTeacherConnectionStatus(runtime.connectionStatus ?? "unknown");
    } catch (_err) {
      get().setTeacherConnectionStatus("unknown");
    }

    if (hasSubscribedTeacherEvents) {
      return;
    }

    try {
      hasSubscribedTeacherEvents = true;

      await onTeacherEndpointApplied((payload) => {
        get().setTeacherMasterEndpoint(payload.endpoint ?? null);
        get().setTeacherConnectionStatus("connecting");
      });

      await onWsConnected(() => {
        get().setTeacherConnectionStatus("connected");
      });

      await onWsDisconnected(() => {
        get().setTeacherConnectionStatus("disconnected");
      });
    } catch (_err) {
      hasSubscribedTeacherEvents = false;
    }
  },
}));

export default useDeviceStore;
