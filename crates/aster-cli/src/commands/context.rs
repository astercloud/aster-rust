use anyhow::Result;
use aster::context::ContextService;

pub fn handle_context_abstract(uri: &str, trace: bool) -> Result<()> {
    let service = ContextService::default();
    let result = service.abstract_content_with_trace(uri)?;
    println!("{}", result.document.content);
    if trace {
        println!("\n--- trace ---");
        for step in result.trace {
            println!("[{}] {}", step.stage, step.detail);
        }
    }
    Ok(())
}

pub fn handle_context_overview(uri: &str, trace: bool) -> Result<()> {
    let service = ContextService::default();
    let result = service.overview_content_with_trace(uri)?;
    println!("{}", result.document.content);
    if trace {
        println!("\n--- trace ---");
        for step in result.trace {
            println!("[{}] {}", step.stage, step.detail);
        }
    }
    Ok(())
}

pub fn handle_context_read(uri: &str, trace: bool) -> Result<()> {
    let service = ContextService::default();
    let result = service.detail_content_with_trace(uri)?;
    println!("{}", result.document.content);
    if trace {
        println!("\n--- trace ---");
        for step in result.trace {
            println!("[{}] {}", step.stage, step.detail);
        }
    }
    Ok(())
}

pub fn handle_context_status(json: bool) -> Result<()> {
    let service = ContextService::default();
    let status = service.status()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!("Context Root: {}", status.root_dir.display());
        println!("Root Exists: {}", status.root_exists);
        for namespace in status.namespaces {
            println!(
                "- {}: exists={}, files={}, dirs={}, path={}",
                namespace.namespace,
                namespace.exists,
                namespace.file_count,
                namespace.dir_count,
                namespace.path.display()
            );
        }
    }
    Ok(())
}
