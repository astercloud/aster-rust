# 诊断和健康检查模块

提供系统健康检查、故障排除功能。

## 文件索引

| 文件 | 说明 |
|------|------|
| `mod.rs` | 模块导出 |
| `checker.rs` | 诊断检查器：Git、Ripgrep、内存、环境变量等检查 |
| `report.rs` | 诊断报告：生成和格式化报告 |

## 核心功能

### DiagnosticChecker
- Git 可用性检查
- Ripgrep 可用性检查
- 磁盘空间检查
- 文件权限检查
- 内存使用检查
- 环境变量检查

### DiagnosticReport
- 生成完整诊断报告
- 支持 JSON 输出
- 详细模式显示系统信息

## 使用示例

```rust
use aster::diagnostics::{run_diagnostics, quick_health_check, DiagnosticReport, DiagnosticOptions};

// 快速健康检查
let (healthy, issues) = quick_health_check();
if !healthy {
    for issue in issues {
        println!("问题: {}", issue);
    }
}

// 完整诊断报告
let options = DiagnosticOptions { verbose: true, ..Default::default() };
let report = DiagnosticReport::generate(&options);
println!("{}", format_diagnostic_report(&report, &options));
```


