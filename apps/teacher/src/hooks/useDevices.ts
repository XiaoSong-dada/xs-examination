import { useCallback, useEffect, useMemo, useState } from "react";
import type {
  DeviceListItem,
  IDeviceCreate,
  IDeviceEditor,
  UseDeviceListResult,
} from "@/types/main";
import {
  createDevice as create,
  deleteDevice as remove,
  getDeviceList,
  updateDevice as update,
} from "@/services/deviceService";
import { deepClone } from "@/utils/utils";

export function useDeviceList(): UseDeviceListResult {
  const [loading, setLoading] = useState(false);
  const [allDevices, setAllDevices] = useState<DeviceListItem[]>([]);
  const [inputIpKeyword, setInputIpKeyword] = useState("");
  const [inputNameKeyword, setInputNameKeyword] = useState("");
  const [appliedIpKeyword, setAppliedIpKeyword] = useState("");
  const [appliedNameKeyword, setAppliedNameKeyword] = useState("");

  const search = useCallback(() => {
    setAppliedIpKeyword(inputIpKeyword);
    setAppliedNameKeyword(inputNameKeyword);
  }, [inputIpKeyword, inputNameKeyword]);

  const reset = useCallback(() => {
    setInputIpKeyword("");
    setInputNameKeyword("");
    setAppliedIpKeyword("");
    setAppliedNameKeyword("");
  }, []);

  const dataSource = useMemo(() => allDevices, [allDevices]);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const list = await getDeviceList({
        ip: appliedIpKeyword.trim() || undefined,
        name: appliedNameKeyword.trim() || undefined,
      });
      setAllDevices(list);
    } catch (error) {
      console.error("[useDeviceList] 获取设备列表失败", error);
      setAllDevices([]);
    } finally {
      setLoading(false);
    }
  }, [appliedIpKeyword, appliedNameKeyword]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const createDevice = useCallback(async (data: IDeviceCreate) => {
    try {
      await create(data);
      await refresh();
      return true;
    } catch (error) {
      console.error("创建设备失败", error);
      return false;
    }
  }, [refresh]);

  const updateDevice = useCallback(async (data: IDeviceEditor) => {
    try {
      await update(data);
      await refresh();
      return true;
    } catch (error) {
      console.error("更新设备失败", error);
      return false;
    }
  }, [refresh]);

  const deleteDevice = useCallback(async (id: string) => {
    try {
      await remove(id);
      await refresh();
      return true;
    } catch (error) {
      console.error("删除设备失败", error);
      return false;
    }
  }, [refresh]);

  return {
    loading,
    inputIpKeyword,
    inputNameKeyword,
    appliedIpKeyword,
    appliedNameKeyword,
    setInputIpKeyword,
    setInputNameKeyword,
    search,
    reset,
    dataSource,
    refresh,
    createDevice,
    updateDevice,
    deleteDevice,
  };
}

const defaultCreateDeviceData: IDeviceEditor = {
  id: "",
  ip: "",
  name: "",
};

export function useDeviceModal() {
  const [visible, setVisible] = useState(false);
  const [modalTitle, setModalTitle] = useState("新增设备");
  const [formData, setFormData] = useState<IDeviceEditor | null>(null);

  const openCreate = useCallback(() => {
    setModalTitle("新增设备");
    setFormData(deepClone(defaultCreateDeviceData));
    setVisible(true);
  }, []);

  const openEdit = useCallback((data: IDeviceEditor) => {
    setModalTitle("编辑设备");
    setFormData(data);
    setVisible(true);
  }, []);

  const close = useCallback(() => {
    setVisible(false);
  }, []);

  return {
    modalTitle,
    visible,
    formData,
    setFormData,
    openCreate,
    openEdit,
    close,
  } as const;
}
