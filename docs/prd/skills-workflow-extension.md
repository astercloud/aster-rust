# Skills 系统扩展方案：Workflow 执行引擎

> 版本：1.0.0  
> 日期：2026-02-05  
> 状态：草案

## 1. 背景与目标

### 1.1 现状分析

aster-rust 已有基础的 Skills 系统：
- `types.rs` - 定义 `SkillDefinition`、`SkillFrontmatter` 等类型
- `loader.rs` - 从 SKILL.md 文件加载 Skill
- `registry.rs` - 全局注册表，支持多目录优先级
- `tool.rs` - 作为 MCP Tool 暴露 `loadSkill`

**当前能力**：
- ✅ 文件系统加载（User/Project/Plugin 三级）
- ✅ Frontmatter 解析（name, description, model, allowed-tools 等）
- ✅ 注册表查找（支持命名空间）
- ❌ **缺失**：执行引擎（只能加载，不能执行）
- ❌ **缺失**：Workflow 多步骤支持
- ❌ **缺失**：Provider 绑定

### 1.2 目标

扩展 aster-rust Skills 系统，支持：

1. **三种执行模式**：
   - `prompt` - 单次对话，注入 System Prompt（现有能力）
   - `workflow` - 多步骤工作流，确定性高
   - `agent` - 多轮迭代探索（未来）

2. **Provider 绑定**：Skill 可指定使用的 LLM Provider

3. **执行引擎**：提供 `SkillExecutor` trait，应用层实现具体调用


## 2. 竞品分析

### 2.1 DeepChat Skills

**特点**：
- 文件优先设计，基于 `~/.deepchat/skills/` 目录
- 渐进式加载：元数据始终在内存，完整内容按需加载
- 热重载：使用 chokidar 监听文件变化
- 跨工具同步：支持 Claude Code、Cursor、Windsurf、Kiro 双向同步
- 会话级激活：每个对话可激活不同 Skills

**可借鉴**：热重载、会话级激活

### 2.2 Refly Skills

**特点**：
- SaaS 平台级设计，基于 LangGraph 状态机
- Skill Registry 中央注册表
- 版本控制、工作流克隆、GitHub 集成
- 多端导出（API、SDK、嵌入）

**可借鉴**：Schema 定义规范、输入/输出规范

### 2.3 CAMEL Workforce

**特点**：
- 双模式执行：`AUTO_DECOMPOSE`（智能分解）、`PIPELINE`（预定义流水线）
- Pipeline 构建器（链式 API）：
  ```python
  workforce.pipeline_add("收集数据")
      .pipeline_fork(["技术分析", "基本面分析"])
      .pipeline_join("生成报告")
      .pipeline_build()
  ```
- 任务依赖管理（DAG 验证）
- 失败恢复策略：RETRY、REPLAN、DECOMPOSE、REASSIGN
- 质量评估：0-100 分，低于 70 分触发恢复

**可借鉴**：Pipeline 模式、失败恢复策略（简化版）


## 3. 架构设计

### 3.1 分层架构

```
┌─────────────────────────────────────────────────────────┐
│                    应用层 (ProxyCast 等)                 │
│  - 实现 LlmProvider trait                               │
│  - 实现 ExecutionCallback（UI 进度展示）                 │
│  - 凭证池管理（应用特有）                                │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                  aster-rust (框架层)                     │
│  - Skills 类型定义、加载、注册                           │
│  - Workflow 执行引擎                                    │
│  - SkillExecutor trait                                  │
│  - 变量插值、依赖排序                                    │
└─────────────────────────────────────────────────────────┘
```

### 3.2 模块结构

```
crates/aster/src/skills/
├── mod.rs           # 模块导出
├── types.rs         # 类型定义 [扩展]
├── loader.rs        # 文件加载 [扩展]
├── registry.rs      # 注册表 [现有]
├── tool.rs          # MCP Tool [现有]
├── executor.rs      # 执行引擎 [新增]
└── workflow.rs      # 工作流处理 [新增]
```


## 4. 数据结构设计

### 4.1 扩展 SkillFrontmatter

```rust
// types.rs

/// Skill frontmatter 元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    // === 现有字段 ===
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "allowed-tools")]
    pub allowed_tools: Option<String>,
    #[serde(rename = "argument-hint")]
    pub argument_hint: Option<String>,
    #[serde(rename = "when-to-use")]
    pub when_to_use: Option<String>,
    pub version: Option<String>,
    pub model: Option<String>,
    #[serde(rename = "user-invocable")]
    pub user_invocable: Option<String>,
    #[serde(rename = "disable-model-invocation")]
    pub disable_model_invocation: Option<String>,
    
    // === 新增字段 ===
    /// 执行模式: prompt | workflow | agent
    #[serde(rename = "execution-mode")]
    pub execution_mode: Option<String>,
    
    /// Provider 绑定（如 "openai", "anthropic", "kiro"）
    pub provider: Option<String>,
    
    /// 工作流定义（仅 workflow 模式）
    pub workflow: Option<WorkflowDefinition>,
}
```

### 4.2 执行模式枚举

```rust
/// Skill 执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillExecutionMode {
    /// 单次对话，注入 System Prompt
    #[default]
    Prompt,
    /// 多步骤工作流
    Workflow,
    /// 多轮迭代探索（未来）
    Agent,
}

impl SkillExecutionMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "workflow" => Self::Workflow,
            "agent" => Self::Agent,
            _ => Self::Prompt,
        }
    }
}
```


### 4.3 工作流定义

```rust
/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// 步骤 ID（唯一标识）
    pub id: String,
    
    /// 步骤名称（用于显示）
    pub name: String,
    
    /// 提示词模板，支持变量插值 ${var_name}
    pub prompt: String,
    
    /// 输入变量引用（可选）
    /// 如 "${previous_output}" 或 "${step_id.output}"
    #[serde(default)]
    pub input: Option<String>,
    
    /// 输出变量名
    pub output: String,
    
    /// 依赖的步骤 ID 列表
    #[serde(default)]
    pub dependencies: Vec<String>,
    
    /// 是否可并行执行（与同级无依赖步骤）
    #[serde(default)]
    pub parallel: bool,
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// 步骤列表
    pub steps: Vec<WorkflowStep>,
    
    /// 失败重试次数（默认 2）
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    
    /// 失败时是否继续执行后续步骤
    #[serde(default)]
    pub continue_on_failure: bool,
}

fn default_max_retries() -> u32 { 2 }
```

### 4.4 扩展 SkillDefinition

```rust
/// Skill 定义（扩展）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    // === 现有字段 ===
    pub skill_name: String,
    pub display_name: String,
    pub description: String,
    pub has_user_specified_description: bool,
    pub markdown_content: String,
    pub allowed_tools: Option<Vec<String>>,
    pub argument_hint: Option<String>,
    pub when_to_use: Option<String>,
    pub version: Option<String>,
    pub model: Option<String>,
    pub disable_model_invocation: bool,
    pub user_invocable: bool,
    pub source: SkillSource,
    pub base_dir: PathBuf,
    pub file_path: PathBuf,
    pub supporting_files: Vec<PathBuf>,
    
    // === 新增字段 ===
    /// 执行模式
    pub execution_mode: SkillExecutionMode,
    
    /// Provider 绑定
    pub provider: Option<String>,
    
    /// 工作流定义（仅 workflow 模式）
    pub workflow: Option<WorkflowDefinition>,
}
```


## 5. 执行引擎设计

### 5.1 核心 Trait

```rust
// executor.rs

use async_trait::async_trait;

/// LLM Provider trait（应用层实现）
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 发送聊天请求
    async fn chat(
        &self,
        system_prompt: &str,
        user_message: &str,
        model: Option<&str>,
    ) -> Result<String, SkillError>;
    
    /// 流式聊天（可选实现）
    async fn chat_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
        model: Option<&str>,
        callback: Box<dyn Fn(&str) + Send>,
    ) -> Result<String, SkillError> {
        // 默认实现：回退到非流式
        self.chat(system_prompt, user_message, model).await
    }
}

/// 执行回调 trait（应用层实现，用于 UI 进度展示）
pub trait ExecutionCallback: Send + Sync {
    /// 步骤开始
    fn on_step_start(&self, step_id: &str, step_name: &str, total_steps: usize);
    
    /// 步骤完成
    fn on_step_complete(&self, step_id: &str, output: &str);
    
    /// 步骤失败
    fn on_step_error(&self, step_id: &str, error: &str, will_retry: bool);
    
    /// 整体完成
    fn on_complete(&self, success: bool, final_output: Option<&str>);
}

/// 空回调实现（用于无 UI 场景）
pub struct NoopCallback;

impl ExecutionCallback for NoopCallback {
    fn on_step_start(&self, _: &str, _: &str, _: usize) {}
    fn on_step_complete(&self, _: &str, _: &str) {}
    fn on_step_error(&self, _: &str, _: &str, _: bool) {}
    fn on_complete(&self, _: bool, _: Option<&str>) {}
}
```

### 5.2 执行器实现

```rust
/// Skill 执行器
pub struct SkillExecutor<P: LlmProvider> {
    provider: P,
}

impl<P: LlmProvider> SkillExecutor<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
    
    /// 执行 Skill
    pub async fn execute(
        &self,
        skill: &SkillDefinition,
        input: &str,
        callback: Option<&dyn ExecutionCallback>,
    ) -> Result<SkillExecutionResult, SkillError> {
        let cb = callback.unwrap_or(&NoopCallback);
        
        match skill.execution_mode {
            SkillExecutionMode::Prompt => {
                self.execute_prompt_mode(skill, input, cb).await
            }
            SkillExecutionMode::Workflow => {
                self.execute_workflow_mode(skill, input, cb).await
            }
            SkillExecutionMode::Agent => {
                Err(SkillError::NotImplemented("Agent 模式尚未实现".into()))
            }
        }
    }
}
```


### 5.3 Prompt 模式执行

```rust
impl<P: LlmProvider> SkillExecutor<P> {
    async fn execute_prompt_mode(
        &self,
        skill: &SkillDefinition,
        input: &str,
        callback: &dyn ExecutionCallback,
    ) -> Result<SkillExecutionResult, SkillError> {
        callback.on_step_start("prompt", "执行提示词", 1);
        
        let system_prompt = &skill.markdown_content;
        let model = skill.model.as_deref();
        
        match self.provider.chat(system_prompt, input, model).await {
            Ok(output) => {
                callback.on_step_complete("prompt", &output);
                callback.on_complete(true, Some(&output));
                
                Ok(SkillExecutionResult {
                    success: true,
                    output: Some(output),
                    error: None,
                    steps_completed: vec![],
                    command_name: Some(skill.skill_name.clone()),
                    allowed_tools: skill.allowed_tools.clone(),
                    model: skill.model.clone(),
                })
            }
            Err(e) => {
                let error_msg = e.to_string();
                callback.on_step_error("prompt", &error_msg, false);
                callback.on_complete(false, None);
                
                Ok(SkillExecutionResult {
                    success: false,
                    output: None,
                    error: Some(error_msg),
                    ..Default::default()
                })
            }
        }
    }
}
```

### 5.4 Workflow 模式执行

```rust
impl<P: LlmProvider> SkillExecutor<P> {
    async fn execute_workflow_mode(
        &self,
        skill: &SkillDefinition,
        input: &str,
        callback: &dyn ExecutionCallback,
    ) -> Result<SkillExecutionResult, SkillError> {
        let workflow = skill.workflow.as_ref()
            .ok_or_else(|| SkillError::InvalidConfig(
                "Workflow 模式需要定义 workflow".into()
            ))?;
        
        // 初始化上下文
        let mut context: HashMap<String, String> = HashMap::new();
        context.insert("user_input".to_string(), input.to_string());
        
        // 拓扑排序
        let sorted_steps = self.topological_sort(&workflow.steps)?;
        let total_steps = sorted_steps.len();
        let mut completed_steps = Vec::new();
        
        for step in sorted_steps {
            callback.on_step_start(&step.id, &step.name, total_steps);
            
            // 变量插值
            let prompt = self.interpolate_variables(&step.prompt, &context);
            let model = skill.model.as_deref();
            
            // 执行步骤（带重试）
            let result = self.execute_step_with_retry(
                &prompt,
                model,
                workflow.max_retries,
                &step.id,
                callback,
            ).await;
            
            match result {
                Ok(output) => {
                    context.insert(step.output.clone(), output.clone());
                    completed_steps.push(StepResult {
                        step_id: step.id.clone(),
                        step_name: step.name.clone(),
                        output,
                        success: true,
                        error: None,
                    });
                    callback.on_step_complete(&step.id, 
                        context.get(&step.output).unwrap());
                }
                Err(e) if workflow.continue_on_failure => {
                    let error_msg = e.to_string();
                    completed_steps.push(StepResult {
                        step_id: step.id.clone(),
                        step_name: step.name.clone(),
                        output: String::new(),
                        success: false,
                        error: Some(error_msg),
                    });
                }
                Err(e) => {
                    callback.on_complete(false, None);
                    return Err(e);
                }
            }
        }
        
        // 返回最后一个步骤的输出
        let final_output = completed_steps.last()
            .filter(|s| s.success)
            .map(|s| s.output.clone());
        
        callback.on_complete(true, final_output.as_deref());
        
        Ok(SkillExecutionResult {
            success: true,
            output: final_output,
            error: None,
            steps_completed: completed_steps,
            command_name: Some(skill.skill_name.clone()),
            allowed_tools: skill.allowed_tools.clone(),
            model: skill.model.clone(),
        })
    }
}
```


## 6. 工作流处理

### 6.1 变量插值

```rust
// workflow.rs

/// 变量插值
/// 支持格式：${var_name}、${step_id.output}
pub fn interpolate_variables(
    template: &str,
    context: &HashMap<String, String>,
) -> String {
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
    
    re.replace_all(template, |caps: &regex::Captures| {
        let var_name = &caps[1];
        context.get(var_name)
            .cloned()
            .unwrap_or_else(|| format!("${{{}}}", var_name))
    }).to_string()
}
```

### 6.2 拓扑排序

```rust
/// 拓扑排序（确保依赖顺序）
pub fn topological_sort(
    steps: &[WorkflowStep],
) -> Result<Vec<&WorkflowStep>, SkillError> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut step_map: HashMap<&str, &WorkflowStep> = HashMap::new();
    
    // 初始化
    for step in steps {
        in_degree.insert(&step.id, step.dependencies.len());
        step_map.insert(&step.id, step);
        graph.entry(&step.id).or_default();
        
        for dep in &step.dependencies {
            graph.entry(dep.as_str()).or_default().push(&step.id);
        }
    }
    
    // BFS
    let mut queue: VecDeque<&str> = in_degree.iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    
    let mut result = Vec::new();
    
    while let Some(id) = queue.pop_front() {
        if let Some(step) = step_map.get(id) {
            result.push(*step);
        }
        
        if let Some(neighbors) = graph.get(id) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }
    
    // 检测循环依赖
    if result.len() != steps.len() {
        return Err(SkillError::InvalidConfig(
            "工作流存在循环依赖".into()
        ));
    }
    
    Ok(result)
}
```

### 6.3 步骤重试

```rust
impl<P: LlmProvider> SkillExecutor<P> {
    async fn execute_step_with_retry(
        &self,
        prompt: &str,
        model: Option<&str>,
        max_retries: u32,
        step_id: &str,
        callback: &dyn ExecutionCallback,
    ) -> Result<String, SkillError> {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match self.provider.chat("", prompt, model).await {
                Ok(output) => return Ok(output),
                Err(e) => {
                    let will_retry = attempt < max_retries;
                    callback.on_step_error(step_id, &e.to_string(), will_retry);
                    last_error = Some(e);
                    
                    if will_retry {
                        // 指数退避
                        let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| 
            SkillError::ExecutionFailed("未知错误".into())
        ))
    }
}
```


## 7. SKILL.md 格式示例

### 7.1 Prompt 模式（简单）

```yaml
---
name: code-reviewer
description: 代码审查助手
provider: anthropic
model: claude-sonnet-4-20250514
execution-mode: prompt
allowed-tools: Read, Write
---

你是一个专业的代码审查专家。请审查用户提供的代码，关注：

1. **代码质量**：可读性、命名规范、代码结构
2. **潜在问题**：Bug、安全漏洞、边界条件
3. **性能优化**：算法效率、资源使用
4. **最佳实践**：设计模式、SOLID 原则

输出格式：
- 问题列表（按严重程度排序）
- 改进建议
- 代码示例（如适用）
```

### 7.2 Workflow 模式（创作场景）

```yaml
---
name: video-script-generator
description: 从选题到完整脚本的自动化流程
version: "1.0.0"
provider: anthropic
model: claude-sonnet-4-20250514
execution-mode: workflow

workflow:
  max_retries: 2
  continue_on_failure: false
  steps:
    - id: analyze_topic
      name: 选题分析
      prompt: |
        分析用户提供的选题「${user_input}」：
        
        请从以下维度分析：
        1. 目标受众是谁？
        2. 核心卖点是什么？
        3. 竞品内容分析
        4. 差异化角度建议
        
        输出 JSON 格式：
        {
          "audience": "目标受众描述",
          "selling_points": ["卖点1", "卖点2"],
          "competitors": ["竞品1", "竞品2"],
          "unique_angle": "差异化角度"
        }
      output: topic_analysis
      
    - id: generate_outline
      name: 生成大纲
      prompt: |
        基于选题分析结果：
        ${topic_analysis}
        
        生成视频大纲，包含：
        - 开场 Hook（前 3 秒抓住注意力）
        - 主体内容（3-5 个核心要点）
        - 结尾 CTA（引导行动）
        
        每个部分标注预计时长。
      input: "${topic_analysis}"
      output: outline
      dependencies: [analyze_topic]
      
    - id: write_script
      name: 撰写脚本
      prompt: |
        根据大纲：
        ${outline}
        
        撰写完整的口播脚本，要求：
        - 口语化表达，避免书面语
        - 每段控制在 30-60 秒
        - 标注情绪和节奏（如 [激动]、[停顿]）
        - 加入互动引导（如"你们觉得呢？"）
      input: "${outline}"
      output: script
      dependencies: [generate_outline]
      
    - id: generate_shots
      name: 生成分镜
      prompt: |
        根据脚本：
        ${script}
        
        生成分镜表，格式：
        | 镜号 | 时长 | 画面描述 | 台词摘要 | 转场 |
        |------|------|----------|----------|------|
        | 1    | 3s   | ...      | ...      | 切   |
      input: "${script}"
      output: shots
      dependencies: [write_script]
---

# 视频脚本生成器

这是一个完整的视频脚本创作工作流，从选题分析到最终分镜。

## 使用方法

输入你的视频选题，例如：
- "如何在 30 天内学会一门新技能"
- "程序员必备的 10 个效率工具"

## 输出内容

1. 选题分析报告
2. 视频大纲
3. 完整口播脚本
4. 分镜表
```


## 8. 应用层集成示例

### 8.1 ProxyCast 集成

```rust
// proxycast/src-tauri/src/skills/mod.rs

use aster::skills::{
    SkillExecutor, ExecutionCallback, LlmProvider,
    SkillDefinition, SkillExecutionResult, SkillError,
};

/// ProxyCast 的 LLM Provider 实现
pub struct ProxyCastProvider {
    provider_pool: Arc<ProviderPoolService>,
    credential_pool_id: Option<String>,
}

#[async_trait]
impl LlmProvider for ProxyCastProvider {
    async fn chat(
        &self,
        system_prompt: &str,
        user_message: &str,
        model: Option<&str>,
    ) -> Result<String, SkillError> {
        let credential = self.provider_pool
            .get_healthy_credential(self.credential_pool_id.as_deref())
            .await
            .map_err(|e| SkillError::ProviderError(e.to_string()))?;
        
        // 调用实际的 LLM API
        // ...
    }
}

/// ProxyCast 的执行回调（用于前端进度展示）
pub struct TauriCallback {
    window: tauri::Window,
}

impl ExecutionCallback for TauriCallback {
    fn on_step_start(&self, step_id: &str, step_name: &str, total: usize) {
        self.window.emit("skill:step_start", json!({
            "step_id": step_id,
            "step_name": step_name,
            "total_steps": total,
        })).ok();
    }
    
    fn on_step_complete(&self, step_id: &str, output: &str) {
        self.window.emit("skill:step_complete", json!({
            "step_id": step_id,
            "output": output,
        })).ok();
    }
    
    fn on_step_error(&self, step_id: &str, error: &str, will_retry: bool) {
        self.window.emit("skill:step_error", json!({
            "step_id": step_id,
            "error": error,
            "will_retry": will_retry,
        })).ok();
    }
    
    fn on_complete(&self, success: bool, final_output: Option<&str>) {
        self.window.emit("skill:complete", json!({
            "success": success,
            "output": final_output,
        })).ok();
    }
}
```

### 8.2 Tauri 命令

```rust
/// 执行 Skill
#[tauri::command]
pub async fn execute_skill(
    skill_name: String,
    input: String,
    credential_pool_id: Option<String>,
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) -> Result<SkillExecutionResult, String> {
    // 1. 从 aster-rust 获取 Skill 定义
    let registry = aster::skills::global_registry();
    let skill = {
        let r = registry.read().map_err(|e| e.to_string())?;
        r.find(&skill_name)
            .ok_or_else(|| format!("Skill '{}' 未找到", skill_name))?
            .clone()
    };
    
    // 2. 创建 Provider（使用 ProxyCast 的凭证池）
    let provider = ProxyCastProvider {
        provider_pool: state.provider_pool.clone(),
        credential_pool_id,
    };
    
    // 3. 创建执行器和回调
    let executor = SkillExecutor::new(provider);
    let callback = TauriCallback { window };
    
    // 4. 执行
    executor.execute(&skill, &input, Some(&callback))
        .await
        .map_err(|e| e.to_string())
}

/// 列出所有 Skills
#[tauri::command]
pub async fn list_skills() -> Result<Vec<SkillSummary>, String> {
    let registry = aster::skills::global_registry();
    let r = registry.read().map_err(|e| e.to_string())?;
    
    Ok(r.get_user_invocable()
        .iter()
        .map(|s| SkillSummary {
            name: s.skill_name.clone(),
            display_name: s.display_name.clone(),
            description: s.description.clone(),
            execution_mode: s.execution_mode,
            provider: s.provider.clone(),
        })
        .collect())
}
```


## 9. 实现路线图

### Phase 1：基础扩展（1-2 天）

**目标**：扩展类型定义，支持 Workflow 解析

**任务**：
- [ ] 扩展 `SkillFrontmatter`，添加 `execution-mode`、`provider`、`workflow` 字段
- [ ] 添加 `SkillExecutionMode` 枚举
- [ ] 添加 `WorkflowDefinition`、`WorkflowStep` 类型
- [ ] 扩展 `SkillDefinition`，添加新字段
- [ ] 更新 `loader.rs`，解析新的 frontmatter 字段
- [ ] 添加单元测试

**文件变更**：
- `crates/aster/src/skills/types.rs` [修改]
- `crates/aster/src/skills/loader.rs` [修改]

### Phase 2：执行引擎（2-3 天）

**目标**：实现 `SkillExecutor` 和 `LlmProvider` trait

**任务**：
- [ ] 创建 `executor.rs` 模块
- [ ] 定义 `LlmProvider` trait
- [ ] 定义 `ExecutionCallback` trait
- [ ] 实现 `SkillExecutor` 结构体
- [ ] 实现 Prompt 模式执行
- [ ] 添加集成测试（使用 Mock Provider）

**文件变更**：
- `crates/aster/src/skills/executor.rs` [新增]
- `crates/aster/src/skills/mod.rs` [修改]

### Phase 3：Workflow 执行（2-3 天）

**目标**：实现 Workflow 模式的完整执行

**任务**：
- [ ] 创建 `workflow.rs` 模块
- [ ] 实现变量插值 `interpolate_variables()`
- [ ] 实现拓扑排序 `topological_sort()`
- [ ] 实现步骤重试逻辑
- [ ] 实现 Workflow 模式执行
- [ ] 添加 Workflow 相关测试

**文件变更**：
- `crates/aster/src/skills/workflow.rs` [新增]
- `crates/aster/src/skills/executor.rs` [修改]

### Phase 4：错误处理与优化（1-2 天）

**目标**：完善错误处理，添加日志

**任务**：
- [ ] 定义 `SkillError` 错误类型
- [ ] 添加 tracing 日志
- [ ] 处理边界情况（空 workflow、循环依赖等）
- [ ] 性能优化（并行步骤执行）
- [ ] 文档完善

**文件变更**：
- `crates/aster/src/skills/error.rs` [新增]
- 各模块添加日志


## 10. 与其他框架对比

| 特性 | aster-rust (现有) | aster-rust (扩展后) | DeepChat | Refly | CAMEL |
|------|-------------------|---------------------|----------|-------|-------|
| 存储方式 | 文件系统 | 文件系统 | 文件系统 | 数据库 | 内存 |
| 执行模式 | ❌ | Prompt + Workflow | Prompt | Workflow | Auto + Pipeline |
| Provider 绑定 | ❌ | ✅ | ❌ | ✅ | ✅ |
| 工作流支持 | ❌ | ✅ | ❌ | ✅ | ✅ |
| 失败重试 | ❌ | ✅（简化版） | ❌ | ❌ | ✅（完整） |
| 热重载 | ❌ | 可扩展 | ✅ | ❌ | ❌ |
| 并行执行 | ❌ | Phase 4 | ❌ | ✅ | ✅ |
| 质量评估 | ❌ | 未来 | ❌ | ❌ | ✅ |

## 11. 设计决策

### 11.1 为什么选择 Trait 而非具体实现？

**决策**：`LlmProvider` 和 `ExecutionCallback` 定义为 trait

**原因**：
1. **解耦**：框架层不依赖具体的 LLM SDK
2. **灵活性**：应用层可自由选择 Provider 实现
3. **可测试**：便于使用 Mock 进行单元测试
4. **复用**：不同应用可共享执行引擎

### 11.2 为什么不实现完整的 CAMEL 恢复策略？

**决策**：只实现简单的重试机制

**原因**：
1. **复杂度**：CAMEL 的 REPLAN、DECOMPOSE、REASSIGN 需要额外的 Agent 协调
2. **场景适配**：ProxyCast 主要是创作场景，确定性工作流为主
3. **渐进式**：先实现基础能力，后续按需扩展

### 11.3 为什么 Workflow 定义在 Frontmatter 中？

**决策**：Workflow 作为 YAML frontmatter 的一部分

**原因**：
1. **一致性**：与现有 SKILL.md 格式保持一致
2. **可读性**：YAML 格式易于人工编辑
3. **版本控制**：单文件便于 Git 管理
4. **兼容性**：不影响现有 Prompt 模式的 Skills

## 12. 未来扩展

### 12.1 Agent 模式（v2.0）

多轮迭代探索模式，适合不确定性高的任务：
- 自主决定下一步行动
- 支持工具调用
- 支持人工介入

### 12.2 并行步骤执行（v1.1）

当多个步骤无依赖关系时，并行执行：
```yaml
workflow:
  steps:
    - id: step_a
      parallel: true
    - id: step_b
      parallel: true
    - id: step_c
      dependencies: [step_a, step_b]
```

### 12.3 条件分支（v1.2）

根据上一步输出决定执行路径：
```yaml
workflow:
  steps:
    - id: analyze
      output: analysis
    - id: path_a
      condition: "${analysis.type} == 'technical'"
    - id: path_b
      condition: "${analysis.type} == 'creative'"
```

### 12.4 热重载（v1.1）

监听 Skills 目录变化，自动重新加载：
```rust
pub fn watch_skills_directory(
    registry: SharedSkillRegistry,
    callback: impl Fn(&str) + Send + 'static,
) -> notify::RecommendedWatcher;
```

---

## 附录 A：完整类型定义

详见 `crates/aster/src/skills/types.rs`

## 附录 B：测试用例

详见 `crates/aster/src/skills/tests/`
