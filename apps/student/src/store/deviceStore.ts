import { create } from "zustand";
import { DeviceStore } from "@/types/main";

export const useDeviceStore = create<DeviceStore>((set) => ({
  ip: null,
  assignedStudent: null,
  setIp: (ip) => set({ ip }),
  setAssignedStudent: (assignedStudent) => set({ assignedStudent }),
}));

export default useDeviceStore;
