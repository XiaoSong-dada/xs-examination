---
description: "Use when editing websocket protocol, shared message contracts, cross-end TypeScript interfaces, answer sync payloads, or packages/shared-types. Covers protocol shape, message naming, and student-facing data safety."
name: "Protocol And Shared Types"
applyTo: "packages/shared-types/src/**"
---

# 协议与共享类型规范

- 前后端共享的 TypeScript 协议与接口统一定义在 `packages/shared-types/src/`，不要在 teacher 或 student 前端各自重复定义跨端协议结构。
- WebSocket 消息结构应与 `packages/shared-types/src/protocol.ts` 中的共享定义保持一致；新增消息类型时，先补共享类型，再补两端实现。
- 消息类型命名保持全大写下划线风格，例如 `EXAM_START`、`ANSWER_SYNC`。
- 任何面向学生端的接口、协议或消息都不得暴露题目标准答案，例如 `questions.answer` 一类字段不能进入学生端可见载荷。
