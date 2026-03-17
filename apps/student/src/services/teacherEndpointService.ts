import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  TeacherRuntimeStatus,
  TeacherEndpointAppliedEvent,
  WsConnectionEvent,
} from "@/types/main";

export async function getTeacherRuntimeStatus(): Promise<TeacherRuntimeStatus> {
  return invoke<TeacherRuntimeStatus>("get_teacher_runtime_status");
}

export async function onTeacherEndpointApplied(
  handler: (payload: TeacherEndpointAppliedEvent) => void,
): Promise<UnlistenFn> {
  return listen<TeacherEndpointAppliedEvent>("teacher_endpoint_applied", (event) => {
    handler(event.payload);
  });
}

export async function onWsConnected(
  handler: (payload: WsConnectionEvent) => void,
): Promise<UnlistenFn> {
  return listen<WsConnectionEvent>("ws_connected", (event) => {
    handler(event.payload);
  });
}

export async function onWsDisconnected(
  handler: (payload: WsConnectionEvent) => void,
): Promise<UnlistenFn> {
  return listen<WsConnectionEvent>("ws_disconnected", (event) => {
    handler(event.payload);
  });
}
