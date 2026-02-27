# 集成测试计划

## 测试策略

### 1. 单元测试 (已完成)
- error.rs 单元测试
- performance.rs 单元测试

### 2. E2E API 测试 (已创建)
```bash
# 启动服务器
./bin/core-server &

# 运行测试
cargo test --package core-engine --test e2e_api_test
```

### 3. 集成测试场景

| 场景 | 描述 | 预期结果 |
|------|------|----------|
| T1 | 启动服务器 + 启动浏览器 | 成功 |
| T2 | 访问页面 + sense | 返回语义树 |
| T3 | 执行 click 操作 | DOM 更新 |
| T4 | 执行 type 操作 | 文本输入 |
| T5 | 页面导航 | 新语义树 |
| T6 | 错误处理 | 正确错误码 |
| T7 | 并发请求 | 正确响应 |
| T8 | 资源清理 | 无泄漏 |

## 部署检查清单

- [ ] 二进制文件可执行
- [ ] 依赖库完整
- [ ] 配置文件正确
- [ ] 日志输出正常
- [ ] 端口可访问
- [ ] Chrome 可启动

## 生产环境检查

```bash
# 1. 检查二进制
file bin/core-server
ls -lh bin/

# 2. 检查依赖
ldd bin/core-server  # Linux
otool -L bin/core-server  # macOS

# 3. 运行测试
./bin/core-server &
sleep 2
curl http://127.0.0.1:9222/health
```
