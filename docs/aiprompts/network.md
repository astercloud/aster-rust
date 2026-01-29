# Network 网络模块

提供代理、超时、重试等网络功能。

## 模块结构

```
network/
├── proxy.rs    # 代理配置
├── retry.rs    # 重试逻辑
└── timeout.rs  # 超时处理
```

## 代理配置

支持 HTTP/HTTPS/SOCKS 代理：
- 环境变量配置
- 配置文件配置
- 自动检测系统代理

## 超时处理

- 连接超时
- 读取超时
- 总请求超时

## 重试逻辑

- 指数退避
- 可配置重试次数
- 错误类型过滤

## 源码位置

`crates/aster/src/network/`
