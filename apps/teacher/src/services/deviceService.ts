import type { DeviceListItem, IDeviceCreate, IDeviceEditor } from "@/types/main";
import { invoke } from "@tauri-apps/api/core";

export interface GetDeviceListParams {
  ip?: string;
  name?: string;
}

export interface DiscoveredDeviceItem {
  ip: string;
}

export interface ReplaceDevicesPayload {
  devices: DiscoveredDeviceItem[];
}

export async function getDeviceList(
  params?: GetDeviceListParams,
): Promise<DeviceListItem[]> {
  return invoke<DeviceListItem[]>("get_devices", {
    payload: {
      ip: params?.ip,
      name: params?.name,
    },
  });
}

export async function getDeviceById(id: string): Promise<DeviceListItem> {
  return invoke<DeviceListItem>("get_device_by_id", { payload: { id } });
}

export async function createDevice(data: IDeviceCreate) {
  return invoke("create_device", { payload: data });
}

export async function updateDevice(data: IDeviceEditor) {
  return invoke("update_device", { payload: data });
}

export async function deleteDevice(id: string) {
  return invoke("delete_device", { payload: { id } });
}

export async function discoverStudentDevices(): Promise<DiscoveredDeviceItem[]> {
  return invoke<DiscoveredDeviceItem[]>("discover_student_devices");
}

export async function replaceDevicesByDiscovery(
  payload: ReplaceDevicesPayload,
): Promise<DeviceListItem[]> {
  return invoke<DeviceListItem[]>("replace_devices_by_discovery", { payload });
}
