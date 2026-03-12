import { Button, Form, Input, message, Modal, Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useRef, useState } from "react";

import { useDeviceList, useDeviceModal } from "@/hooks/useDevices";
import {
  discoverStudentDevices,
  getDeviceById,
  replaceDevicesByDiscovery,
} from "@/services/deviceService";
import { useTableHeight } from "@/hooks/useTableHeight";
import type { IDeviceCreate, IDeviceEditor, DeviceListItem } from "@/types/main";

export function DevicesPage() {
  const {
    loading,
    inputIpKeyword,
    inputNameKeyword,
    setInputIpKeyword,
    setInputNameKeyword,
    search,
    reset,
    dataSource,
    refresh,
    createDevice,
    updateDevice,
    deleteDevice,
  } = useDeviceList();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);
  const tableHeight = useTableHeight(containerRef, toolbarRef);

  const deviceModal = useDeviceModal();
  const [discoveryVisible, setDiscoveryVisible] = useState(false);
  const [discovering, setDiscovering] = useState(false);
  const [replacing, setReplacing] = useState(false);
  const [discoveredDevices, setDiscoveredDevices] = useState<Array<{ ip: string }>>([]);
  const [form] = Form.useForm();

  useEffect(() => {
    if (!deviceModal.visible) return;
    if (deviceModal.formData) {
      form.setFieldsValue(deviceModal.formData as any);
    } else {
      form.resetFields();
    }
  }, [deviceModal.formData, deviceModal.visible, form]);

  const handleDelete = (id: string) => {
    Modal.confirm({
      title: "确认删除",
      content: "删除后不可恢复，是否继续？",
      okText: "删除",
      okButtonProps: { danger: true },
      cancelText: "取消",
      onOk: async () => {
        const ok = await deleteDevice(id);
        if (ok) {
          message.success("删除成功");
          await refresh();
        } else {
          message.error("删除失败");
        }
      },
    });
  };

  const handleEdit = async (id: string) => {
    try {
      const detail = await getDeviceById(id);
      deviceModal.openEdit({
        id: detail.id,
        ip: detail.ip,
        name: detail.name,
      });
    } catch (error) {
      console.error("获取设备详情失败", error);
      message.error("获取设备详情失败");
    }
  };

  const columns: ColumnsType<DeviceListItem> = [
    {
      title: "IP",
      dataIndex: "ip",
      key: "ip",
      width: 240,
    },
    {
      title: "设备名称",
      dataIndex: "name",
      key: "name",
      width: 260,
    },
    {
      title: "操作",
      dataIndex: "id",
      key: "id",
      width: 140,
      fixed: "right",
      render: (id: string) => (
        <div className="flex gap-2">
          <Button
            type="link"
            onClick={() => void handleEdit(id)}
          >
            修改
          </Button>
          <Button type="link" danger onClick={() => handleDelete(id)}>
            删除
          </Button>
        </div>
      ),
    },
  ];

  const onFinish = async (values: IDeviceEditor) => {
    const payload: IDeviceEditor = {
      id: values.id,
      ip: values.ip?.trim(),
      name: values.name?.trim(),
    };

    if (payload.id) {
      const ok = await updateDevice(payload);
      if (ok) {
        message.success("更新成功");
        deviceModal.close();
        await refresh();
      } else {
        message.error("更新失败");
      }
      return;
    }

    const createPayload: IDeviceCreate = {
      ip: payload.ip,
      name: payload.name,
    };

    const ok = await createDevice(createPayload);
    if (ok) {
      message.success("创建成功");
      deviceModal.close();
      await refresh();
    } else {
      message.error("创建失败");
    }
  };

  const handleSearchDevices = async () => {
    setDiscovering(true);
    try {
      const list = await discoverStudentDevices();
      setDiscoveredDevices(list);
      setDiscoveryVisible(true);
      if (list.length === 0) {
        message.warning("未发现可用设备");
      }
    } catch (error) {
      console.error("搜索设备失败", error);
      message.error("搜索设备失败");
    } finally {
      setDiscovering(false);
    }
  };

  const handleConfirmReplace = () => {
    Modal.confirm({
      title: "确认替换设备列表",
      content: "此操作将先清空原有设备列表，是否继续？",
      okText: "确认替换",
      okButtonProps: { danger: true },
      cancelText: "取消",
      onOk: async () => {
        setReplacing(true);
        try {
          await replaceDevicesByDiscovery({ devices: discoveredDevices });
          message.success(`设备替换成功，共 ${discoveredDevices.length} 台`);
          setDiscoveryVisible(false);
          await refresh();
        } catch (error) {
          console.error("替换设备列表失败", error);
          message.error("替换设备列表失败");
        } finally {
          setReplacing(false);
        }
      },
    });
  };

  const discoveryColumns: ColumnsType<{ ip: string }> = [
    {
      title: "IP",
      dataIndex: "ip",
      key: "ip",
    },
  ];

  return (
    <div className="space-y-4 h-full">
      <div
        ref={containerRef}
        className="bg-white rounded-lg border border-gray-200 p-4 h-full"
      >
        <div
          ref={toolbarRef}
          className="bg-white rounded-lg flex flex-col gap-5 pb-4 w-full"
        >
          <div className="flex gap-4">
            <div className="flex-1 max-w-sm">
              <Input
                value={inputIpKeyword}
                allowClear
                placeholder="按 IP 模糊查询"
                onChange={(event) => setInputIpKeyword(event.target.value)}
                onPressEnter={search}
              />
            </div>
            <div className="flex-1 max-w-sm">
              <Input
                value={inputNameKeyword}
                allowClear
                placeholder="按设备名称模糊查询"
                onChange={(event) => setInputNameKeyword(event.target.value)}
                onPressEnter={search}
              />
            </div>
            <div className="flex items-center gap-2">
              <Button type="primary" onClick={search}>
                搜索
              </Button>
              <Button onClick={reset}>重置</Button>
            </div>
          </div>
          <div className="flex gap-2">
            <Button type="primary" onClick={deviceModal.openCreate}>
              新增设备
            </Button>
            <Button loading={discovering} onClick={() => void handleSearchDevices()}>
              搜索设备
            </Button>
          </div>
        </div>

        <Table<DeviceListItem>
          rowKey="id"
          loading={loading}
          dataSource={dataSource}
          columns={columns}
          pagination={false}
          scroll={{ y: tableHeight }}
        />
      </div>

      <Modal
        title={deviceModal.modalTitle}
        open={deviceModal.visible}
        onCancel={deviceModal.close}
        onOk={() => form.submit()}
        okText="确认"
        cancelText="取消"
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={onFinish}
          initialValues={deviceModal.formData as any}
        >
          <Form.Item name="id" hidden>
            <Input />
          </Form.Item>

          <Form.Item
            name="ip"
            label="IP"
            rules={[{ required: true, message: "请输入设备 IP" }]}
          >
            <Input />
          </Form.Item>

          <Form.Item
            name="name"
            label="设备名称"
            rules={[{ required: true, message: "请输入设备名称" }]}
          >
            <Input />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title="搜索到的设备"
        open={discoveryVisible}
        onCancel={() => setDiscoveryVisible(false)}
        onOk={handleConfirmReplace}
        okText="确认"
        cancelText="取消"
        confirmLoading={replacing}
        width={760}
      >
        <Table<{ ip: string }>
          rowKey="ip"
          pagination={false}
          dataSource={discoveredDevices}
          columns={discoveryColumns}
          locale={{ emptyText: "未发现设备" }}
          scroll={{ y: 360 }}
        />
      </Modal>
    </div>
  );
}
