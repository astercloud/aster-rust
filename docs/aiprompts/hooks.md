# Hooks 系统

支持在工具调用前后执行自定义脚本或回调。

## 模块结构

```
hooks/
├── executor.rs  # Hook 执行器
├── loader.rs    # Hook 加载器
├── registry.rs  # Hook 注册表
└── types.rs     # 类型定义
```

## 核心组件

- `HookExecutor` - 执行 hook 脚本
- `HookLoader` - 从配置加载 hooks
- `HookRegistry` - 管理已注册的 hooks

## Hook 类型

1. **Pre-hooks** - 工具调用前执行
2. **Post-hooks** - 工具调用后执行
3. **Error hooks** - 错误发生时执行

## 配置示例

```yaml
hooks:
  pre_tool:
    - name: "validate_input"
      script: "./scripts/validate.sh"
  post_tool:
    - name: "log_result"
      script: "./scripts/log.sh"
```

## 使用场景

- 输入验证
- 结果日志记录
- 自定义通知
- 安全检查

## 源码位置

`crates/aster/src/hooks/`
