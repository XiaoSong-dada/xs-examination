---
description: "Use when editing documentation in doc/, especially e2e chain docs, implementation plans, or flow descriptions. Covers when to update e2e docs, naming rules, and linking instead of duplicating content."
name: "E2E Docs"
applyTo: "doc/**/*.md"
---

# 文档与 E2E 规范

- 若改动触及已有业务闭环，先阅读对应的 `doc/e2e` 最短链路文档，再开始改代码或改文档。
- 若本次改动改变了链路入口、真实出口、关键持久化落点、主查询来源、运行态事件或页面验证面，必须同步更新对应的 `doc/e2e/*.md`。
- 若本次改动形成新的独立业务闭环，新增一份 `e2e-minimal-<business>-chain.md` 风格的最短链路文档。
- E2E 文档正文优先回答四件事：入口在哪里、真实出口在哪里、最短调用链是什么、哪些内容不属于这条链路。
- 编写或更新文档时，优先链接现有正式文档，不要在多份文档中重复复制同一段设计说明。
