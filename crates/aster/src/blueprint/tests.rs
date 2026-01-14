//! 蓝图系统测试
//!
//! 测试蓝图管理器和任务树管理器的核心功能

use super::*;

#[cfg(test)]
mod blueprint_tests {
    use super::*;

    #[tokio::test]
    async fn test_blueprint_lifecycle() {
        let manager = BlueprintManager::default();

        // 1. 创建蓝图
        let bp = manager
            .create_blueprint("测试蓝图".to_string(), "测试描述".to_string())
            .await
            .unwrap();
        assert_eq!(bp.status, BlueprintStatus::Draft);

        // 2. 添加业务流程
        let process = BusinessProcess {
            id: String::new(),
            name: "用户注册流程".to_string(),
            description: "新用户注册".to_string(),
            process_type: ProcessType::ToBe,
            steps: vec![ProcessStep {
                id: "step1".to_string(),
                order: 1,
                name: "填写信息".to_string(),
                description: "用户填写注册信息".to_string(),
                actor: "用户".to_string(),
                system_action: None,
                user_action: Some("填写表单".to_string()),
                conditions: Vec::new(),
                outcomes: vec!["注册信息".to_string()],
            }],
            actors: vec!["用户".to_string()],
            inputs: vec!["用户信息".to_string()],
            outputs: vec!["用户账号".to_string()],
        };
        manager.add_business_process(&bp.id, process).await.unwrap();

        // 3. 添加系统模块
        let module = SystemModule {
            id: String::new(),
            name: "用户服务".to_string(),
            description: "用户管理服务".to_string(),
            module_type: ModuleType::Backend,
            responsibilities: vec!["用户注册".to_string(), "用户认证".to_string()],
            dependencies: Vec::new(),
            interfaces: Vec::new(),
            tech_stack: Some(vec!["Rust".to_string()]),
            root_path: Some("src/user".to_string()),
        };
        manager.add_module(&bp.id, module).await.unwrap();

        // 4. 获取更新后的蓝图
        let updated_bp = manager.get_blueprint(&bp.id).await.unwrap();
        assert_eq!(updated_bp.business_processes.len(), 1);
        assert_eq!(updated_bp.modules.len(), 1);
    }
}
