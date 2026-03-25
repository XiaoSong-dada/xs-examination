import { create } from "zustand";
import { DeviceStore, TeacherConnectionStatus } from "@/types/main";
import {
  getTeacherRuntimeStatus,
  onTeacherEndpointApplied,
  onWsConnected,
  onWsDisconnected,
} from "@/services/teacherEndpointService";
import {
  getDeviceRuntimeStatus,
  onDeviceIpUpdated,
} from "@/services/deviceService";

let hasSubscribedTeacherEvents = false;
let hasSubscribedDeviceEvents = false;

export const useDeviceStore = create<DeviceStore>((set, get) => ({
  ip: null,
  assignedStudent: null,
  teacherMasterEndpoint: null,
  teacherConnectionStatus: "unknown",

  setIp: (ip) => set({ ip }),
  setAssignedStudent: (assignedStudent) => set({ assignedStudent }),

  setTeacherMasterEndpoint: (ep) => set({ teacherMasterEndpoint: ep }),
  setTeacherConnectionStatus: (s: TeacherConnectionStatus) => set({ teacherConnectionStatus: s }),

  async initDeviceInfo() {
    try {
      const runtime = await getDeviceRuntimeStatus();
      get().setIp(runtime.ip ?? null);
    } catch (_err) {
      get().setIp(null);
    }

    if (hasSubscribedDeviceEvents) {
      return;
    }

    try {
      hasSubscribedDeviceEvents = true;
      await onDeviceIpUpdated((payload) => {
        get().setIp(payload.ip ?? null);
      });
    } catch (_err) {
      hasSubscribedDeviceEvents = false;
    }
  },

  async initTeacherInfo() {
    await get().initDeviceInfo();

    try {
      const runtime = await getTeacherRuntimeStatus();
      get().setTeacherMasterEndpoint(runtime.endpoint ?? null);
      if ((runtime.endpoint ?? null) && runtime.connectionStatus !== "connected") {
        get().setTeacherConnectionStatus("connecting");
      } else {
        get().setTeacherConnectionStatus(runtime.connectionStatus ?? "unknown");
      }
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
        const hasEndpoint = Boolean(get().teacherMasterEndpoint);
        get().setTeacherConnectionStatus(hasEndpoint ? "connecting" : "disconnected");
      });
    } catch (_err) {
      hasSubscribedTeacherEvents = false;
    }
  },
}));

export default useDeviceStore;
