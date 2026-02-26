use anyhow::Result;
use aster::session::{CommitOptions, MemoryCategory, SessionManager};

pub async fn handle_memory_extract(
    session_id: String,
    force: bool,
    max_messages: Option<usize>,
) -> Result<()> {
    let report = SessionManager::commit_session(
        &session_id,
        CommitOptions {
            force,
            max_messages,
        },
    )
    .await?;

    println!("session_id: {}", report.session_id);
    println!("messages_scanned: {}", report.messages_scanned);
    println!("memories_created: {}", report.memories_created);
    println!("memories_merged: {}", report.memories_merged);
    if !report.warnings.is_empty() {
        println!("warnings:");
        for warning in report.warnings {
            println!("- {}", warning);
        }
    }
    Ok(())
}

pub async fn handle_memory_search(
    query: String,
    limit: Option<usize>,
    session_id: Option<String>,
    categories: Option<Vec<String>>,
) -> Result<()> {
    let parsed_categories = categories
        .map(|cats| {
            cats.into_iter()
                .map(|item| item.parse::<MemoryCategory>())
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;

    let results =
        SessionManager::search_memories(&query, limit, session_id.as_deref(), parsed_categories)
            .await?;

    if results.is_empty() {
        println!("No memories found.");
        return Ok(());
    }

    for (index, item) in results.iter().enumerate() {
        println!(
            "{}. [{}] {}",
            index + 1,
            item.record.category,
            item.record.abstract_text
        );
        println!("   session: {}", item.record.session_id);
        println!("   score: {:.4}", item.relevance_score);
    }

    Ok(())
}

pub async fn handle_memory_stats() -> Result<()> {
    let stats = SessionManager::memory_stats().await?;
    let health = SessionManager::memory_health().await?;

    println!("healthy: {}", health.healthy);
    println!("message: {}", health.message);
    println!("total_memories: {}", stats.total_memories);
    println!("total_sessions: {}", stats.total_sessions);
    println!("total_events: {}", stats.total_events);
    println!("total_links: {}", stats.total_links);
    Ok(())
}
