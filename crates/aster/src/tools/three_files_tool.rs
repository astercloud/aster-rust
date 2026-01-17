use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::tools::{Tool, ToolContext, ToolError, ToolResult};

/// 三阶段工作流工具 - 基于 planning-with-files 的核心机制
///
/// 核心理念：文件系统作为持久化记忆 (Context Window = RAM, Filesystem = Disk)
/// 三阶段工作流：Pre-Action → Action → Post-Action
///
/// 功能：
/// 1. 自动化上下文工程 - 防止目标漂移和上下文丢失
/// 2. 错误学习机制 - 3次错误协议，永不重复失败
/// 3. 2-Action 规则 - 每2次视觉操作后立即保存发现
/// 4. 阶段完成验证 - 确保工作流完整性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeStageWorkflowTool {
    name: String,
    description: String,
}

impl Default for ThreeStageWorkflowTool {
    fn default() -> Self {
        Self {
            name: "three_stage_workflow".to_string(),
            description: "Implements three-stage workflow pattern for complex tasks. Manages task_plan.md, findings.md, and progress.md with automated context engineering and error learning.".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParams {
    pub action: String,
    pub project_name: Option<String>,
    pub phase_update: Option<PhaseUpdate>,
    pub finding: Option<String>,
    pub progress_entry: Option<String>,
    pub error_info: Option<ErrorInfo>,
    pub decision: Option<DecisionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseUpdate {
    pub phase_number: u32,
    pub status: String, // "pending", "in_progress", "complete"
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub error_description: String,
    pub attempt_number: u32,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionInfo {
    pub decision: String,
    pub rationale: String,
}

#[async_trait]
impl Tool for ThreeStageWorkflowTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "init_workflow",
                        "pre_action_check",
                        "post_action_update",
                        "update_phase",
                        "add_finding",
                        "add_progress",
                        "log_error",
                        "log_decision",
                        "check_completion",
                        "apply_2action_rule"
                    ],
                    "description": "Workflow action: init_workflow (create files), pre_action_check (read plan before action), post_action_update (update after action), update_phase (change phase status), add_finding (save discovery), add_progress (log action), log_error (track error), log_decision (record decision), check_completion (verify all phases complete), apply_2action_rule (save findings after 2 visual operations)"
                },
                "project_name": {
                    "type": "string",
                    "description": "Name of the project (used for init_workflow action)"
                },
                "phase_update": {
                    "type": "object",
                    "properties": {
                        "phase_number": {
                            "type": "integer",
                            "description": "Phase number to update (1-5)"
                        },
                        "status": {
                            "type": "string",
                            "enum": ["pending", "in_progress", "complete"],
                            "description": "New status for the phase"
                        },
                        "notes": {
                            "type": "string",
                            "description": "Optional notes about the phase update"
                        }
                    },
                    "required": ["phase_number", "status"]
                },
                "finding": {
                    "type": "string",
                    "description": "Finding or discovery to add to findings.md (use after visual operations)"
                },
                "progress_entry": {
                    "type": "string",
                    "description": "Progress entry to add to progress.md"
                },
                "error_info": {
                    "type": "object",
                    "properties": {
                        "error_description": {
                            "type": "string",
                            "description": "Description of the error encountered"
                        },
                        "attempt_number": {
                            "type": "integer",
                            "description": "Which attempt this was (1, 2, 3)"
                        },
                        "resolution": {
                            "type": "string",
                            "description": "How the error was resolved (if resolved)"
                        }
                    },
                    "required": ["error_description", "attempt_number"]
                },
                "decision": {
                    "type": "object",
                    "properties": {
                        "decision": {
                            "type": "string",
                            "description": "The decision made"
                        },
                        "rationale": {
                            "type": "string",
                            "description": "The reasoning behind the decision"
                        }
                    },
                    "required": ["decision", "rationale"]
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let params: WorkflowParams =
            serde_json::from_value(params).map_err(|e| ToolError::invalid_params(e.to_string()))?;

        match params.action.as_str() {
            "init_workflow" => self.init_workflow(params.project_name.as_deref()),
            "pre_action_check" => self.pre_action_check(),
            "post_action_update" => self.post_action_update(),
            "update_phase" => {
                if let Some(phase_update) = params.phase_update {
                    self.update_phase(phase_update)
                } else {
                    Err(ToolError::invalid_params(
                        "phase_update required for update_phase action",
                    ))
                }
            }
            "add_finding" => {
                if let Some(finding) = params.finding {
                    self.add_finding(&finding)
                } else {
                    Err(ToolError::invalid_params(
                        "finding required for add_finding action",
                    ))
                }
            }
            "add_progress" => {
                if let Some(progress_entry) = params.progress_entry {
                    self.add_progress(&progress_entry)
                } else {
                    Err(ToolError::invalid_params(
                        "progress_entry required for add_progress action",
                    ))
                }
            }
            "log_error" => {
                if let Some(error_info) = params.error_info {
                    self.log_error(error_info)
                } else {
                    Err(ToolError::invalid_params(
                        "error_info required for log_error action",
                    ))
                }
            }
            "log_decision" => {
                if let Some(decision) = params.decision {
                    self.log_decision(decision)
                } else {
                    Err(ToolError::invalid_params(
                        "decision required for log_decision action",
                    ))
                }
            }
            "check_completion" => self.check_completion(),
            "apply_2action_rule" => {
                if let Some(finding) = params.finding {
                    self.apply_2action_rule(&finding)
                } else {
                    Err(ToolError::invalid_params(
                        "finding required for apply_2action_rule action",
                    ))
                }
            }
            _ => Err(ToolError::invalid_params(format!(
                "Unknown action: {}",
                params.action
            ))),
        }
    }
}

impl ThreeStageWorkflowTool {
    /// 初始化三阶段工作流文件
    fn init_workflow(&self, project_name: Option<&str>) -> Result<ToolResult, ToolError> {
        let project_name = project_name.unwrap_or("project");
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

        // 创建 task_plan.md - 核心规划文件
        let task_plan_content = format!(
            r#"# Task Plan: {}

## Goal
[One sentence describing the end state]

## Current Phase
Phase 1

## Phases

### Phase 1: Requirements & Discovery
- [ ] Understand user intent and requirements
- [ ] Identify constraints and dependencies
- [ ] Document findings in findings.md
- **Status:** in_progress

### Phase 2: Planning & Structure
- [ ] Define technical approach and architecture
- [ ] Create project structure if needed
- [ ] Document key decisions with rationale
- **Status:** pending

### Phase 3: Implementation
- [ ] Execute the plan step by step
- [ ] Write code to files before executing
- [ ] Test incrementally and document results
- **Status:** pending

### Phase 4: Testing & Verification
- [ ] Verify all requirements are met
- [ ] Document test results in progress.md
- [ ] Fix any issues found and log resolutions
- **Status:** pending

### Phase 5: Delivery & Completion
- [ ] Review all output files and deliverables
- [ ] Ensure completeness and quality
- [ ] Deliver final results to user
- **Status:** pending

## Key Questions
1. [Important question to answer during the task]
2. [Another key question that guides decisions]

## Decisions Made
| Decision | Rationale |
|----------|-----------|
|          |           |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
|       | 1       |            |

## Notes
- **CRITICAL**: Re-read this plan before major decisions (attention manipulation)
- **2-Action Rule**: Save findings after every 2 view/browser operations
- **3-Strike Protocol**: Never repeat the same failed action 3 times
- Update phase status: pending → in_progress → complete
"#,
            project_name
        );

        // 创建 findings.md - 研究发现存储
        let findings_content = r#"# Findings & Research

## Requirements Analysis
-

## Research Findings
-

## Technical Decisions
| Decision | Rationale | Impact |
|----------|-----------|--------|

## Issues & Solutions
| Issue | Root Cause | Resolution |
|-------|------------|------------|

## Resources & References
-

## Key Insights
-
"#;

        // 创建 progress.md - 会话日志
        let progress_content = format!(
            r#"# Progress Log

## Session: {}

### Current Status
- **Phase:** 1 - Requirements & Discovery
- **Started:** {}
- **Visual Operations Count:** 0

### Actions Taken
-

### Test Results
| Test | Expected | Actual | Status |
|------|----------|--------|--------|

### Error Log
| Time | Error | Attempt | Resolution |
|------|-------|---------|------------|

### Context Refreshes
| Time | Reason | Action |
|------|--------|--------|
"#,
            date, date
        );

        // 写入文件
        fs::write("task_plan.md", task_plan_content)?;
        fs::write("findings.md", findings_content)?;
        fs::write("progress.md", progress_content)?;

        Ok(ToolResult::success("✅ Three-stage workflow initialized! Created task_plan.md, findings.md, progress.md with automated context engineering.")
            .with_metadata("files_created", serde_json::json!(["task_plan.md", "findings.md", "progress.md"]))
            .with_metadata("project_name", serde_json::json!(project_name))
            .with_metadata("workflow_stage", serde_json::json!("initialized")))
    }

    /// Pre-Action 阶段：在执行重要操作前刷新上下文
    fn pre_action_check(&self) -> Result<ToolResult, ToolError> {
        if !Path::new("task_plan.md").exists() {
            return Err(ToolError::execution_failed(
                "task_plan.md not found. Run init_workflow first.",
            ));
        }

        let content = fs::read_to_string("task_plan.md")?;

        // 提取关键信息
        let goal = self.extract_goal(&content);
        let current_phase = self.extract_current_phase(&content);
        let pending_tasks = self.extract_pending_tasks(&content);

        // 记录上下文刷新
        let _ = self.log_context_refresh("Pre-action context refresh");

        Ok(ToolResult::success(format!(
            "🔄 Pre-Action Context Refresh:\n\n**Goal:** {}\n**Current Phase:** {}\n**Pending Tasks:**\n{}\n\n⚠️ Keep these goals in mind for the next action!",
            goal,
            current_phase,
            pending_tasks.join("\n")
        ))
        .with_metadata("goal", serde_json::json!(goal))
        .with_metadata("current_phase", serde_json::json!(current_phase))
        .with_metadata("pending_tasks", serde_json::json!(pending_tasks))
        .with_metadata("workflow_stage", serde_json::json!("pre_action")))
    }

    /// Post-Action 阶段：提醒更新状态
    fn post_action_update(&self) -> Result<ToolResult, ToolError> {
        Ok(ToolResult::success("📝 Post-Action Reminder:\n\n1. Did this action complete a phase? Update task_plan.md status\n2. Any new findings? Add to findings.md\n3. Any errors encountered? Log them for learning\n4. Update progress.md with what was accomplished")
            .with_metadata("workflow_stage", serde_json::json!("post_action"))
            .with_metadata("reminder_type", serde_json::json!("status_update")))
    }

    /// 应用 2-Action 规则：每2次视觉操作后保存发现
    fn apply_2action_rule(&self, finding: &str) -> Result<ToolResult, ToolError> {
        // 更新视觉操作计数
        let _ = self.increment_visual_operations();

        // 添加发现到 findings.md
        self.add_finding(finding)?;

        Ok(ToolResult::success(format!(
            "🎯 2-Action Rule Applied: Saved finding after visual operations\n\nFinding: {}",
            finding
        ))
        .with_metadata("rule_applied", serde_json::json!("2_action_rule"))
        .with_metadata("finding", serde_json::json!(finding)))
    }

    /// 记录错误信息（3次错误协议）
    fn log_error(&self, error_info: ErrorInfo) -> Result<ToolResult, ToolError> {
        if !Path::new("task_plan.md").exists() {
            return Err(ToolError::execution_failed(
                "task_plan.md not found. Run init_workflow first.",
            ));
        }

        let mut content = fs::read_to_string("task_plan.md")?;

        // 添加错误到错误表格
        let resolution = error_info.resolution.as_deref().unwrap_or("In progress");
        let error_entry = format!(
            "| {} | {} | {} |",
            error_info.error_description, error_info.attempt_number, resolution
        );

        if let Some(pos) = content.find("## Errors Encountered\n| Error | Attempt | Resolution |\n|-------|---------|------------|") {
            let insert_pos = pos + "## Errors Encountered\n| Error | Attempt | Resolution |\n|-------|---------|------------|\n".len();
            content.insert_str(insert_pos, &format!("{}\n", error_entry));
        }

        fs::write("task_plan.md", content)?;

        // 同时记录到 progress.md
        let _ = self.log_error_to_progress(&error_info);

        let warning = if error_info.attempt_number >= 3 {
            "\n⚠️ WARNING: 3rd attempt! Consider escalating to user or changing approach completely."
        } else {
            ""
        };

        Ok(ToolResult::success(format!(
            "🚨 Error Logged (Attempt {}): {}{}",
            error_info.attempt_number, error_info.error_description, warning
        ))
        .with_metadata("error_logged", serde_json::json!(true))
        .with_metadata(
            "attempt_number",
            serde_json::json!(error_info.attempt_number),
        )
        .with_metadata(
            "needs_escalation",
            serde_json::json!(error_info.attempt_number >= 3),
        ))
    }

    /// 记录决策信息
    fn log_decision(&self, decision_info: DecisionInfo) -> Result<ToolResult, ToolError> {
        if !Path::new("task_plan.md").exists() {
            return Err(ToolError::execution_failed(
                "task_plan.md not found. Run init_workflow first.",
            ));
        }

        let mut content = fs::read_to_string("task_plan.md")?;

        // 添加决策到决策表格
        let decision_entry = format!(
            "| {} | {} |",
            decision_info.decision, decision_info.rationale
        );

        if let Some(pos) =
            content.find("## Decisions Made\n| Decision | Rationale |\n|----------|-----------|")
        {
            let insert_pos = pos
                + "## Decisions Made\n| Decision | Rationale |\n|----------|-----------|\n".len();
            content.insert_str(insert_pos, &format!("{}\n", decision_entry));
        }

        fs::write("task_plan.md", content)?;

        Ok(ToolResult::success(format!(
            "📋 Decision Logged: {} (Rationale: {})",
            decision_info.decision, decision_info.rationale
        ))
        .with_metadata("decision", serde_json::json!(decision_info.decision))
        .with_metadata("rationale", serde_json::json!(decision_info.rationale)))
    }

    /// 检查任务完成状态
    fn check_completion(&self) -> Result<ToolResult, ToolError> {
        if !Path::new("task_plan.md").exists() {
            return Err(ToolError::execution_failed(
                "task_plan.md not found. Run init_workflow first.",
            ));
        }

        let content = fs::read_to_string("task_plan.md")?;

        // 统计阶段状态
        let total_phases = content.matches("### Phase").count();
        let complete_phases = content.matches("**Status:** complete").count();
        let in_progress_phases = content.matches("**Status:** in_progress").count();
        let pending_phases = content.matches("**Status:** pending").count();

        let is_complete = complete_phases == total_phases && total_phases > 0;
        let completion_percentage = if total_phases > 0 {
            (complete_phases as f64 / total_phases as f64 * 100.0) as u32
        } else {
            0
        };

        let status_message = if is_complete {
            "🎉 ALL PHASES COMPLETE! Task ready for delivery."
        } else {
            "⏳ Task in progress. Do not stop until all phases are complete."
        };

        Ok(ToolResult::success(format!(
            "� Task Completion Status:\n\n{}\n\n📈 Progress: {}% ({}/{} phases complete)\n📋 Breakdown: {} complete, {} in progress, {} pending",
            status_message,
            completion_percentage,
            complete_phases,
            total_phases,
            complete_phases,
            in_progress_phases,
            pending_phases
        ))
        .with_metadata("total_phases", serde_json::json!(total_phases))
        .with_metadata("complete_phases", serde_json::json!(complete_phases))
        .with_metadata("in_progress_phases", serde_json::json!(in_progress_phases))
        .with_metadata("pending_phases", serde_json::json!(pending_phases))
        .with_metadata("is_complete", serde_json::json!(is_complete))
        .with_metadata("completion_percentage", serde_json::json!(completion_percentage)))
    }

    // 辅助方法实现
    fn update_phase(&self, phase_update: PhaseUpdate) -> Result<ToolResult, ToolError> {
        if !Path::new("task_plan.md").exists() {
            return Err(ToolError::execution_failed(
                "task_plan.md not found. Run init_workflow first.",
            ));
        }

        let content = fs::read_to_string("task_plan.md")?;

        // 更新阶段状态 - 使用字符串替换方式
        let phase_pattern = format!("### Phase {}", phase_update.phase_number);
        let mut updated_content = content.clone();

        // 找到目标阶段并更新状态
        if let Some(phase_start) = updated_content.find(&phase_pattern) {
            // 找到下一个阶段或文件结尾
            let search_start = phase_start + phase_pattern.len();
            let next_phase_pos = updated_content
                .get(search_start..)
                .and_then(|s| s.find("### Phase"))
                .map(|pos| search_start + pos)
                .unwrap_or(updated_content.len());

            let phase_section = updated_content
                .get(phase_start..next_phase_pos)
                .unwrap_or("");

            // 替换状态行
            let old_status_pattern = "**Status:**";
            if let Some(status_pos) = phase_section.find(old_status_pattern) {
                let status_line_start = phase_start + status_pos;
                let status_line_end = updated_content
                    .get(status_line_start..)
                    .and_then(|s| s.find('\n'))
                    .map(|pos| status_line_start + pos)
                    .unwrap_or(updated_content.len());

                let new_status_line = format!("- **Status:** {}", phase_update.status);
                updated_content.replace_range(status_line_start..status_line_end, &new_status_line);
            }
        }
        fs::write("task_plan.md", updated_content)?;

        // 更新当前阶段指示器
        if phase_update.status == "in_progress" {
            let _ = self.update_current_phase(phase_update.phase_number);
        }

        Ok(ToolResult::success(format!(
            "✅ Phase {} status updated to: {}",
            phase_update.phase_number, phase_update.status
        ))
        .with_metadata("phase", serde_json::json!(phase_update.phase_number))
        .with_metadata("status", serde_json::json!(phase_update.status)))
    }

    fn add_finding(&self, finding: &str) -> Result<ToolResult, ToolError> {
        if !Path::new("findings.md").exists() {
            return Err(ToolError::execution_failed(
                "findings.md not found. Run init_workflow first.",
            ));
        }

        let mut content = fs::read_to_string("findings.md")?;

        let timestamp = chrono::Utc::now().format("%H:%M").to_string();
        let finding_entry = format!("- [{}] {}", timestamp, finding);

        if let Some(pos) = content.find("## Research Findings\n-") {
            let insert_pos = pos + "## Research Findings\n".len();
            content.insert_str(insert_pos, &format!("{}\n", finding_entry));
        } else {
            content.push_str(&format!("\n{}\n", finding_entry));
        }

        fs::write("findings.md", content)?;

        Ok(
            ToolResult::success(format!("💡 Finding saved: {}", finding))
                .with_metadata("timestamp", serde_json::json!(timestamp))
                .with_metadata("finding", serde_json::json!(finding)),
        )
    }

    fn add_progress(&self, progress_entry: &str) -> Result<ToolResult, ToolError> {
        if !Path::new("progress.md").exists() {
            return Err(ToolError::execution_failed(
                "progress.md not found. Run init_workflow first.",
            ));
        }

        let mut content = fs::read_to_string("progress.md")?;

        let timestamp = chrono::Utc::now().format("%H:%M").to_string();
        let progress_line = format!("- [{}] {}", timestamp, progress_entry);

        if let Some(pos) = content.find("### Actions Taken\n-") {
            let insert_pos = pos + "### Actions Taken\n".len();
            content.insert_str(insert_pos, &format!("{}\n", progress_line));
        } else {
            content.push_str(&format!("\n{}\n", progress_line));
        }

        fs::write("progress.md", content)?;

        Ok(
            ToolResult::success(format!("📝 Progress logged: {}", progress_entry))
                .with_metadata("timestamp", serde_json::json!(timestamp))
                .with_metadata("progress", serde_json::json!(progress_entry)),
        )
    }

    // 私有辅助方法
    fn extract_goal(&self, content: &str) -> String {
        content
            .lines()
            .skip_while(|line| !line.starts_with("## Goal"))
            .nth(1)
            .unwrap_or("[Goal not found]")
            .trim()
            .to_string()
    }

    fn extract_current_phase(&self, content: &str) -> String {
        content
            .lines()
            .skip_while(|line| !line.starts_with("## Current Phase"))
            .nth(1)
            .unwrap_or("Phase 1")
            .trim()
            .to_string()
    }

    fn extract_pending_tasks(&self, content: &str) -> Vec<String> {
        content
            .lines()
            .filter(|line| line.contains("- [ ]"))
            .map(|line| line.trim().to_string())
            .collect()
    }

    fn log_context_refresh(&self, reason: &str) -> Result<(), std::io::Error> {
        if !Path::new("progress.md").exists() {
            return Ok(());
        }

        let mut content = fs::read_to_string("progress.md")?;
        let timestamp = chrono::Utc::now().format("%H:%M").to_string();
        let refresh_entry = format!("| {} | {} | Context refreshed |", timestamp, reason);

        if let Some(pos) = content.find("### Context Refreshes\n| Time | Reason | Action |") {
            let insert_pos = pos + "### Context Refreshes\n| Time | Reason | Action |\n|------|--------|---------|\n".len();
            content.insert_str(insert_pos, &format!("{}\n", refresh_entry));
            fs::write("progress.md", content)?;
        }

        Ok(())
    }

    fn increment_visual_operations(&self) -> Result<(), std::io::Error> {
        if !Path::new("progress.md").exists() {
            return Ok(());
        }

        let content = fs::read_to_string("progress.md")?;

        // 简单实现：在进度文件中记录视觉操作
        let timestamp = chrono::Utc::now().format("%H:%M").to_string();
        let updated_content = content.replace(
            "### Actions Taken\n-",
            &format!(
                "### Actions Taken\n- [{}] Visual operation recorded (2-Action Rule tracking)\n-",
                timestamp
            ),
        );

        fs::write("progress.md", updated_content)?;
        Ok(())
    }

    fn log_error_to_progress(&self, error_info: &ErrorInfo) -> Result<(), std::io::Error> {
        if !Path::new("progress.md").exists() {
            return Ok(());
        }

        let mut content = fs::read_to_string("progress.md")?;
        let timestamp = chrono::Utc::now().format("%H:%M").to_string();
        let error_entry = format!(
            "| {} | {} | {} | {} |",
            timestamp,
            error_info.error_description,
            error_info.attempt_number,
            error_info.resolution.as_deref().unwrap_or("In progress")
        );

        if let Some(pos) = content.find("### Error Log\n| Time | Error | Attempt | Resolution |") {
            let insert_pos = pos + "### Error Log\n| Time | Error | Attempt | Resolution |\n|------|-------|---------|------------|\n".len();
            content.insert_str(insert_pos, &format!("{}\n", error_entry));
            fs::write("progress.md", content)?;
        }

        Ok(())
    }

    fn update_current_phase(&self, phase_number: u32) -> Result<(), std::io::Error> {
        if !Path::new("task_plan.md").exists() {
            return Ok(());
        }

        let content = fs::read_to_string("task_plan.md")?;
        let updated_content = content.replace(
            "## Current Phase\nPhase",
            &format!("## Current Phase\nPhase {}", phase_number),
        );

        fs::write("task_plan.md", updated_content)?;
        Ok(())
    }
}
