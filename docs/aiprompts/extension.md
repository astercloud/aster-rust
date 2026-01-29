# 扩展系统

## 概述

扩展系统管理 MCP 服务器和内置扩展，为 Agent 提供额外能力。

**核心路径**: `crates/aster/src/agents/extension.rs`

## 扩展类型

```rust
pub enum ExtensionConfig {
    Stdio { ... },           // 标准 I/O 进程
    StreamableHttp { ... },  // HTTP 流式传输
    Builtin { ... },         // 内置扩展
    Platform { ... },        // 平台扩展
    Frontend { ... },        // 前端工具
    InlinePython { ... },    // 内联 Python
    Sse { ... },             // SSE (已废弃)
}
```

## Stdio 扩展

通过子进程运行 MCP 服务器：

```rust
ExtensionConfig::Stdio {
    name: String,
    description: String,
    cmd: String,
    args: Vec<String>,
    envs: Envs,
    env_keys: Vec<String>,
    timeout: Option<u64>,
    bundled: Option<bool>,
    available_tools: Vec<String>,
}
```

配置示例：

```yaml
type: stdio
name: filesystem
description: 文件系统访问
cmd: npx
args: ["-y", "@modelcontextprotocol/server-filesystem", "/path"]
timeout: 300
```

## StreamableHttp 扩展

通过 HTTP 连接 MCP 服务器：

```rust
ExtensionConfig::StreamableHttp {
    name: String,
    description: String,
    uri: String,
    envs: Envs,
    headers: HashMap<String, String>,
    timeout: Option<u64>,
}
```

## Builtin 扩展

Aster 内置的 MCP 服务器：

```rust
ExtensionConfig::Builtin {
    name: String,
    display_name: Option<String>,
    description: String,
    timeout: Option<u64>,
}
```

内置扩展列表：
- `developer` - 开发者工具
- `memory` - 记忆服务
- `tutorial` - 教程服务
- `computer-controller` - 计算机控制
- `auto-visualiser` - 自动可视化

## Platform 扩展

运行在 Agent 进程内的扩展：

```rust
pub static PLATFORM_EXTENSIONS: Lazy<HashMap<&str, PlatformExtensionDef>>;
```

| 名称 | 说明 | 默认启用 |
|------|------|----------|
| `todo` | TODO 列表管理 | ✅ |
| `chatrecall` | 对话记忆搜索 | ❌ |
| `extensionmanager` | 扩展管理 | ✅ |
| `skills` | 技能加载 | ✅ |
| `codeexecution` | 代码执行沙箱 | ❌ |

## Frontend 扩展

前端提供的工具：

```rust
ExtensionConfig::Frontend {
    name: String,
    description: String,
    tools: Vec<Tool>,
    instructions: Option<String>,
}
```

## InlinePython 扩展

内联 Python 代码：

```rust
ExtensionConfig::InlinePython {
    name: String,
    description: String,
    code: String,
    timeout: Option<u64>,
    dependencies: Option<Vec<String>>,
}
```

配置示例：

```yaml
type: inline_python
name: data_processor
description: 数据处理工具
code: |
  import json
  def process(data):
      return json.dumps(data)
dependencies:
  - pandas
  - numpy
timeout: 300
```

## 环境变量安全

禁止覆盖的环境变量：

```rust
const DISALLOWED_KEYS: [&str; 31] = [
    "PATH", "LD_PRELOAD", "LD_LIBRARY_PATH",
    "DYLD_INSERT_LIBRARIES", "PYTHONPATH",
    "NODE_OPTIONS", ...
];
```

## 工具可用性

```rust
impl ExtensionConfig {
    pub fn is_tool_available(&self, tool_name: &str) -> bool {
        // 如果 available_tools 为空，所有工具可用
        // 否则只有列出的工具可用
    }
}
```

## 扩展信息

```rust
pub struct ExtensionInfo {
    pub name: String,
    pub instructions: String,
    pub has_resources: bool,
}

pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub permission: Option<PermissionLevel>,
}
```

## CLI 使用

```bash
# 添加 stdio 扩展
aster session --with-extension "npx @mcp/server-filesystem /path"

# 添加 HTTP 扩展
aster session --with-streamable-http-extension "http://localhost:8080"

# 添加内置扩展
aster session --with-builtin developer
aster session --with-builtin developer,memory
```
