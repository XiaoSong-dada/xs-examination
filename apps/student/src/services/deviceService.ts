import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DeviceIpUpdatedEvent, DeviceRuntimeStatus } from "@/types/main";

export async function getDeviceRuntimeStatus(): Promise<DeviceRuntimeStatus> {
  return invoke<DeviceRuntimeStatus>("get_device_runtime_status");
}

export async function onDeviceIpUpdated(
  handler: (payload: DeviceIpUpdatedEvent) => void,
): Promise<UnlistenFn> {
  return listen<DeviceIpUpdatedEvent>("device_ip_updated", (event) => {
    handler(event.payload);
  });
}
