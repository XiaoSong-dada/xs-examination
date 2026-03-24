---
description: "Use when editing websocket protocol, shared message contracts, cross-end payloads, answer sync messages, or transitional shared-type definitions. Covers protocol source-of-truth checks, message naming, and student-facing data safety."
name: "Protocol Transition"
applyTo: "apps/student/src/**, apps/teacher/src/**, apps/student/src-tauri/src/**, apps/teacher/src-tauri/src/**, packages/shared-types/src/**"
---

# 协议过渡规范

- `packages/shared-types` 当前不是现行共享协议的单一事实来源；修改跨端协议前，先核对 teacher/student 两端当前实际使用的 `schemas`、`network`、前端 `services/types` 与正式文档。
- 若任务只是修正现有链路，不要为了抽象统一而新增或恢复对 `packages/shared-types` 的依赖。
- 若确实需要重建跨端共享类型，必须先确认两端存在稳定且重复的协议结构，再由专门计划明确落点后实施。
- 消息类型命名保持全大写下划线风格，例如 `EXAM_START`、`ANSWER_SYNC`。
- 任何面向学生端的接口、协议或消息都不得暴露题目标准答案，例如 `questions.answer` 一类字段不能进入学生端可见载荷。
