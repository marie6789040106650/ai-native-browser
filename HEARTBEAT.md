# HEARTBEAT.md - Auto Task Runner

> ⚠️ 此文件驱动自动任务执行
> 不要手动编辑，使用任务更新接口

## 任务状态

读取 `ai-native-browser/TASK_STATE.md` 获取当前状态。

## 执行规则

1. **读取状态** - 检查 `status` 字段
2. **判断** - 如果 `status == idle`，执行下一个任务
3. **不打断** - 如果 `status == running`，不做任何操作
4. **更新状态** - 执行完成后更新 TASK_STATE.md

## 当前任务 (2026-02-25)

**P1-2: Tauri 编译验证**

```
状态: idle
下一步: cargo tauri build 检查
```

---

## 自动执行流程

```
HEARTBEAT → 读取 TASK_STATE.md 
          → 检查 status
          → 如果 idle → 执行下一个任务 → 更新状态
          → 如果 running → NO_REPLY
```
