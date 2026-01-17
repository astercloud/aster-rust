#!/usr/bin/env python3
"""
测试脚本：验证 aster-rust 增强的多模态 read 工具功能

这个脚本演示了增强后的 read 工具如何处理不同类型的文件：
- 文本文件（带语言检测和分析能力）
- 图片文件（带元数据和 AI 分析提示）
- SVG 文件（带矢量图形分析）
- Jupyter 笔记本（带计算分析）
"""

import json
import tempfile
import os
from pathlib import Path

def create_test_files():
    """创建测试文件"""
    temp_dir = Path(tempfile.mkdtemp())
    
    # 创建 Python 源码文件
    python_file = temp_dir / "example.py"
    python_file.write_text("""#!/usr/bin/env python3
def fibonacci(n):
    \"\"\"计算斐波那契数列\"\"\"
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

if __name__ == "__main__":
    for i in range(10):
        print(f"fib({i}) = {fibonacci(i)}")
""")
    
    # 创建 Rust 源码文件
    rust_file = temp_dir / "example.rs"
    rust_file.write_text("""//! 示例 Rust 代码
//! 演示增强的文件读取功能

use std::collections::HashMap;

/// 用户结构体
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

impl User {
    /// 创建新用户
    pub fn new(id: u64, name: String, email: String) -> Self {
        Self { id, name, email }
    }
}

fn main() {
    let mut users = HashMap::new();
    users.insert(1, User::new(1, "Alice".to_string(), "alice@example.com".to_string()));
    users.insert(2, User::new(2, "Bob".to_string(), "bob@example.com".to_string()));
    
    println!("用户列表:");
    for (id, user) in &users {
        println!("  {}: {} <{}>", id, user.name, user.email);
    }
}
""")
    
    # 创建 JSON 配置文件
    json_file = temp_dir / "config.json"
    json_file.write_text(json.dumps({
        "app_name": "MultiModal Reader Test",
        "version": "1.0.0",
        "features": {
            "image_analysis": True,
            "pdf_processing": True,
            "svg_rendering": True,
            "notebook_analysis": True
        },
        "supported_formats": [
            "png", "jpg", "jpeg", "gif", "webp",
            "pdf", "svg", "ipynb",
            "py", "rs", "js", "ts", "json", "yaml", "md"
        ]
    }, indent=2, ensure_ascii=False))
    
    # 创建 SVG 文件
    svg_file = temp_dir / "diagram.svg"
    svg_file.write_text("""<?xml version="1.0" encoding="UTF-8"?>
<svg width="200" height="200" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:rgb(255,255,0);stop-opacity:1" />
      <stop offset="100%" style="stop-color:rgb(255,0,0);stop-opacity:1" />
    </linearGradient>
  </defs>
  
  <!-- 背景 -->
  <rect width="200" height="200" fill="url(#grad1)" />
  
  <!-- 圆形 -->
  <circle cx="100" cy="100" r="50" fill="blue" opacity="0.7" />
  
  <!-- 文本 -->
  <text x="100" y="105" font-family="Arial" font-size="16" fill="white" text-anchor="middle">
    AI Analysis
  </text>
  
  <!-- 箭头 -->
  <path d="M 50 150 L 100 120 L 150 150" stroke="black" stroke-width="3" fill="none" />
  <polygon points="100,115 105,125 95,125" fill="black" />
</svg>""")
    
    # 创建 Jupyter 笔记本
    notebook_file = temp_dir / "analysis.ipynb"
    notebook_data = {
        "cells": [
            {
                "cell_type": "markdown",
                "metadata": {},
                "source": [
                    "# 数据分析示例\n",
                    "\n",
                    "这个笔记本演示了增强的多模态分析功能。\n"
                ]
            },
            {
                "cell_type": "code",
                "execution_count": 1,
                "metadata": {},
                "outputs": [
                    {
                        "name": "stdout",
                        "output_type": "stream",
                        "text": [
                            "数据加载完成\n",
                            "样本数量: 1000\n"
                        ]
                    }
                ],
                "source": [
                    "import pandas as pd\n",
                    "import numpy as np\n",
                    "import matplotlib.pyplot as plt\n",
                    "\n",
                    "# 生成示例数据\n",
                    "np.random.seed(42)\n",
                    "data = pd.DataFrame({\n",
                    "    'x': np.random.randn(1000),\n",
                    "    'y': np.random.randn(1000) * 2 + 1\n",
                    "})\n",
                    "\n",
                    "print(\"数据加载完成\")\n",
                    "print(f\"样本数量: {len(data)}\")"
                ]
            },
            {
                "cell_type": "code",
                "execution_count": 2,
                "metadata": {},
                "outputs": [],
                "source": [
                    "# 数据可视化\n",
                    "plt.figure(figsize=(10, 6))\n",
                    "plt.scatter(data['x'], data['y'], alpha=0.6)\n",
                    "plt.xlabel('X 值')\n",
                    "plt.ylabel('Y 值')\n",
                    "plt.title('散点图分析')\n",
                    "plt.grid(True, alpha=0.3)\n",
                    "plt.show()"
                ]
            }
        ],
        "metadata": {
            "kernelspec": {
                "display_name": "Python 3",
                "language": "python",
                "name": "python3"
            },
            "language_info": {
                "name": "python",
                "version": "3.8.5"
            }
        },
        "nbformat": 4,
        "nbformat_minor": 4
    }
    
    notebook_file.write_text(json.dumps(notebook_data, indent=2, ensure_ascii=False))
    
    return temp_dir

def main():
    """主函数"""
    print("🚀 创建测试文件...")
    test_dir = create_test_files()
    
    print(f"📁 测试文件已创建在: {test_dir}")
    print("\n📋 创建的文件列表:")
    for file_path in sorted(test_dir.iterdir()):
        print(f"  - {file_path.name} ({file_path.stat().st_size} bytes)")
    
    print(f"\n✅ 测试文件准备完成！")
    print(f"💡 你现在可以使用 aster-rust 的增强 read 工具来分析这些文件：")
    print(f"   例如: aster read {test_dir}/example.py")
    print(f"   例如: aster read {test_dir}/diagram.svg")
    print(f"   例如: aster read {test_dir}/analysis.ipynb")
    
    return test_dir

if __name__ == "__main__":
    main()