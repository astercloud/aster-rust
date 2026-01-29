# Codesign 代码签名

用于签名和验证代码的安全模块。

## 模块结构

```
codesign/
├── keys.rs     # 密钥管理
├── signing.rs  # 签名操作
├── storage.rs  # 存储
└── types.rs    # 类型定义
```

## 功能

- 生成签名密钥对 (Ed25519)
- 对文件内容进行哈希和签名
- 验证文件签名
- 签名缓存和持久化

## 使用场景

- 验证代码完整性
- 防止代码篡改
- 安全审计

## 源码位置

`crates/aster/src/codesign/`
