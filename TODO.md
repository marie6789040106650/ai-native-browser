# AI Native Browser - Todo List

## 项目完善任务

### P0 - 高优先级

| # | 任务 | 状态 | 进度 | 更新日期 |
|---|------|------|------|----------|
| 1 | 实现 CDP/Browser 控制 | ✅ 已完成 | 100% | 2026-02-25 |
| 2 | Inspector 事件绑定 | ✅ 已完成 | 100% | 2026-02-25 |
| 3 | API Server 集成 | ✅ 已完成 | 100% | 2026-02-25 |

### P1 - 中优先级

| # | 任务 | 状态 | 进度 | 更新日期 |
|---|------|------|------|----------|
| 4 | HTML 解析器接入 | ⏳ 待开始 | 0% | 2026-02-25 |
| 5 | Tauri 编译验证 | ⏳ 待开始 | 0% | 2026-02-25 |

### P2 - 低优先级

| # | 任务 | 状态 | 进度 | 更新日期 |
|---|------|------|------|----------|
| 6 | 清理代码警告 | ⏳ 待开始 | 0% | 2026-02-25 |
| 7 | 添加单元测试 | ⏳ 待开始 | 0% | 2026-02-25 |

---

## 完善记录

### 2026-02-25

**P0-1: CDP/Browser 控制** (`ee0eee9`)
- 方法: Chrome subprocess + CDP HTTP API (ureq)

**P0-2: Inspector 事件绑定** (`b85aa84`)
- 方法: request_started(), request_finished(), dom_mutated(), on_navigate(), mark_ready()

**P0-3: API Server 集成** (`509c969`)
- /v1/sense: ✅ CDP + Parser 集成
- /v1/act: ✅ CDP 执行 click/type/scroll
- /v1/browser/start: ✅ 真实启动 Chrome

---

## 当前状态

| 项目 | 状态 |
|------|------|
| 编译 | ✅ 通过 |
| Warnings | 9 个 |
| P0 任务 | ✅ 全部完成 |
| 最新 Commit | `509c969` |
