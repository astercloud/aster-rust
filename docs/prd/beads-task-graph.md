# Beads 创作任务模块 PRD

> `beads/` 是 `blueprint/` 模块的**创作场景扩展**，借鉴 [steveyegge/beads](https://github.com/steveyegge/beads) 的设计理念。

## 1. 背景与定位

### 1.1 模块关系

```
┌─────────────────────────────────────────────────────────────┐
│                        plan/                                 │
│              高层计划、方案对比、需求分析                      │
│                        (已实现)                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      blueprint/                              │
│         任务树、TDD 工作流、检查点、Agent 协调                │
│                   (即将实现，对齐 claude-code-open)           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        beads/                                │
│         创作场景扩展：Molecule 模板、内容产出、跨 Session      │
│                        (本 PRD)                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 blueprint vs beads

| 维度 | blueprint（代码开发） | beads（内容创作） |
|------|----------------------|------------------|
| 任务类型 | 代码任务、测试任务 | 文章、视频、教程、文档 |
| 验证方式 | TDD 循环、测试通过 | 完成标准、字数、质量检查 |
| 产出物 | 代码文件、补丁 | 内容文件、URL |
| 工作流 | 测试→编码→验证 | 大纲→初稿→润色→发布 |
| Agent 模式 | Queen/Worker 协调 | 单 Agent 或简单协作 |

### 1.3 beads 的核心价值

1. **Molecule 模板**：标准化创作工作流（博客、视频脚本、教程）
2. **内容产出追踪**：字数、版本、发布状态
3. **跨 Session 恢复**：用户说"继续写那个系列"，Agent 能找到进度
4. **依赖图谱**：系列内容的顺序依赖（第2篇依赖第1篇）

---

## 2. 前置依赖：blueprint 模块

beads 模块**依赖** blueprint 的以下能力：

```rust
// 复用 blueprint 的核心类型
use crate::blueprint::{
    TaskNode,      // 任务节点
    TaskStatus,    // 任务状态
    TaskTree,      // 任务树
    Checkpoint,    // 检查点
};
```

### 2.1 blueprint 需要提供的扩展点

```rust
// blueprint/types.rs 需要支持扩展
pub struct TaskNode {
    // ... 现有字段 ...
    
    /// 扩展数据（beads 用于存储创作相关信息）
    pub extension: Option<serde_json::Value>,
}

pub enum TaskType {
    // blueprint 原有
    Code,
    Test,
    Review,
    
    // beads 扩展
    Content(ContentType),
}

pub enum ContentType {
    Article,
    Video,
    Tutorial,
    Document,
    Story,
}
```

---

## 3. beads 扩展设计

### 3.1 创作任务扩展

```rust
/// 创作任务扩展数据（存储在 TaskNode.extension）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTaskExtension {
    /// 内容类型
    pub content_type: ContentType,
    
    /// 产出信息
    pub output: Option<ContentOutput>,
    
    /// Molecule 模板 ID（如果使用模板创建）
    pub molecule_id: Option<String>,
    
    /// 创作笔记
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentOutput {
    /// 产出文件路径或 URL
    pub location: String,
    
    /// 字数统计
    pub word_count: Option<u32>,
    
    /// 发布状态
    pub publish_status: PublishStatus,
    
    /// 发布 URL
    pub publish_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublishStatus {
    Draft,
    Ready,
    Published,
    Archived,
}
```

### 3.2 Molecule 创作模板

```rust
/// 创作工作流模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Molecule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content_type: ContentType,
    pub steps: Vec<MoleculeStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoleculeStep {
    pub title: String,
    pub description: String,
    /// 完成标准
    pub acceptance: String,
    /// 预估时间（分钟）
    pub estimated_minutes: Option<u32>,
}
```

### 3.3 内置模板示例

**博客写作**：
```yaml
id: blog-writing
name: 博客写作
content_type: Article
steps:
  - title: 确定主题
    acceptance: 有清晰的标题和一句话摘要
  - title: 撰写大纲
    acceptance: 大纲包含 3-5 个主要部分
  - title: 初稿写作
    acceptance: 字数达到目标，覆盖所有大纲要点
  - title: 润色修改
    acceptance: 通读流畅，无明显错误
  - title: 发布准备
    acceptance: 可以直接发布
```

**视频脚本**：
```yaml
id: video-script
name: 视频脚本
content_type: Video
steps:
  - title: 选题调研
    acceptance: 确定视频主题和目标受众
  - title: 脚本大纲
    acceptance: 完成分段大纲
  - title: 详细脚本
    acceptance: 完成逐字稿
  - title: 分镜设计
    acceptance: 完成画面描述
  - title: 配音文稿
    acceptance: 标注语气和节奏
```

---

## 4. 模块结构

```
aster-rust/crates/aster/src/beads/
├── mod.rs
│
├── types/
│   ├── mod.rs
│   ├── content.rs        // ContentType, ContentOutput, PublishStatus
│   └── extension.rs      // ContentTaskExtension
│
├── molecule/
│   ├── mod.rs
│   ├── template.rs       // Molecule, MoleculeStep
│   ├── builtin.rs        // 内置模板（博客、视频、教程）
│   ├── loader.rs         // 从 YAML 加载自定义模板
│   └── executor.rs       // 从模板生成 TaskTree
│
└── api/
    ├── mod.rs
    ├── create.rs         // 创建创作任务
    ├── resume.rs         // 跨 Session 恢复
    └── publish.rs        // 发布状态管理
```

---

## 5. 核心 API

### 5.1 从模板创建任务树

```rust
impl BeadsService {
    /// 使用 Molecule 模板创建创作任务树
    pub async fn create_from_molecule(
        &self,
        molecule_id: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<TaskTree> {
        // 1. 加载模板
        let molecule = self.get_molecule(molecule_id)?;
        
        // 2. 创建根任务
        let root = TaskNode {
            name: title.to_string(),
            task_type: TaskType::Content(molecule.content_type.clone()),
            extension: Some(serde_json::to_value(ContentTaskExtension {
                content_type: molecule.content_type.clone(),
                molecule_id: Some(molecule_id.to_string()),
                ..Default::default()
            })?),
            ..Default::default()
        };
        
        // 3. 生成步骤子任务
        for (i, step) in molecule.steps.iter().enumerate() {
            // 创建子任务，设置顺序依赖
        }
        
        // 4. 持久化到 blueprint 存储
        self.blueprint.save_tree(&tree).await
    }
}
```

### 5.2 跨 Session 恢复

```rust
impl BeadsService {
    /// 根据关键词恢复之前的创作任务
    pub async fn resume_work(&self, hint: &str) -> Result<Option<TaskNode>> {
        // 1. 搜索匹配的创作任务
        let tasks = self.blueprint.search_tasks(TaskFilter {
            task_type: Some(TaskType::Content(_)),
            name_contains: Some(hint.to_string()),
            status: vec![TaskStatus::Pending, TaskStatus::InProgress],
            ..Default::default()
        }).await?;
        
        // 2. 返回最近的进行中任务
        Ok(tasks.into_iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .next()
            .or_else(|| tasks.into_iter().next()))
    }
}
```

### 5.3 更新发布状态

```rust
impl BeadsService {
    /// 标记内容已发布
    pub async fn mark_published(
        &self,
        task_id: &str,
        publish_url: &str,
    ) -> Result<()> {
        let mut task = self.blueprint.get_task(task_id).await?;
        
        if let Some(ext) = &mut task.extension {
            let mut content_ext: ContentTaskExtension = 
                serde_json::from_value(ext.clone())?;
            
            if let Some(output) = &mut content_ext.output {
                output.publish_status = PublishStatus::Published;
                output.publish_url = Some(publish_url.to_string());
            }
            
            *ext = serde_json::to_value(content_ext)?;
        }
        
        self.blueprint.update_task(&task).await
    }
}
```

---

## 6. 与 Agent 集成

```rust
impl Agent {
    /// 开始创作任务（使用模板）
    pub async fn start_content(&self, template: &str, title: &str) -> Result<String> {
        self.beads.create_from_molecule(template, title, None).await
    }
    
    /// 继续之前的创作
    pub async fn continue_content(&self, hint: &str) -> Result<Option<TaskNode>> {
        self.beads.resume_work(hint).await
    }
    
    /// 获取下一个创作步骤
    pub async fn next_content_step(&self, tree_id: &str) -> Result<Option<TaskNode>> {
        self.blueprint.get_next_ready_task(tree_id).await
    }
}
```

---

## 7. 实现计划

### Phase 0: 等待 blueprint 模块完成

beads 依赖 blueprint 的：
- [ ] TaskNode、TaskTree 基础类型
- [ ] TaskStatus 状态机
- [ ] 任务持久化存储
- [ ] 依赖图和 Ready Work 计算

### Phase 1: 核心类型（0.5 周）

- [ ] `types/content.rs`：ContentType, ContentOutput
- [ ] `types/extension.rs`：ContentTaskExtension
- [ ] 单元测试

### Phase 2: Molecule 模板（1 周）

- [ ] `molecule/template.rs`：模板定义
- [ ] `molecule/builtin.rs`：内置模板
- [ ] `molecule/loader.rs`：YAML 加载
- [ ] `molecule/executor.rs`：生成 TaskTree

### Phase 3: API 与集成（0.5 周）

- [ ] `api/create.rs`：创建创作任务
- [ ] `api/resume.rs`：跨 Session 恢复
- [ ] `api/publish.rs`：发布状态
- [ ] Agent 集成

**总计：约 2 周**（blueprint 完成后）

---

## 8. 参考资料

- [steveyegge/beads](https://github.com/steveyegge/beads) - Molecule 模板设计灵感
- [claude-code-open/blueprint](https://github.com/anthropics/claude-code-open/tree/main/src/blueprint) - 基础 TaskTree 设计
- aster-rust `blueprint/` 模块（即将实现）
