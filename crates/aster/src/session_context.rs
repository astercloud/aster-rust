use crate::conversation::message::ActionRequiredScope;
use futures::{Stream, StreamExt};
use tokio::task_local;

pub const SESSION_ID_HEADER: &str = "aster-session-id";

task_local! {
    pub static SESSION_ID: Option<String>;
}

task_local! {
    pub static ACTION_SCOPE: ActionRequiredScope;
}

pub async fn with_session_id<F>(session_id: Option<String>, f: F) -> F::Output
where
    F: std::future::Future,
{
    if let Some(id) = session_id {
        SESSION_ID.scope(Some(id), f).await
    } else {
        f.await
    }
}

pub fn current_session_id() -> Option<String> {
    SESSION_ID.try_with(|id| id.clone()).ok().flatten()
}

pub async fn with_action_scope<F>(scope: ActionRequiredScope, f: F) -> F::Output
where
    F: std::future::Future,
{
    let session_id = scope.session_id.clone();
    if let Some(id) = session_id {
        SESSION_ID
            .scope(Some(id), ACTION_SCOPE.scope(scope, f))
            .await
    } else {
        ACTION_SCOPE.scope(scope, f).await
    }
}

pub fn current_action_scope() -> Option<ActionRequiredScope> {
    ACTION_SCOPE.try_with(|scope| scope.clone()).ok()
}

pub fn scope_stream<S>(scope: ActionRequiredScope, stream: S) -> impl Stream<Item = S::Item> + Send
where
    S: Stream + Unpin + Send,
{
    futures::stream::unfold((scope, stream), |(scope, mut stream)| async move {
        let next = with_action_scope(scope.clone(), async { stream.next().await }).await;
        next.map(|item| (item, (scope, stream)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_id_available_when_set() {
        with_session_id(Some("test-session-123".to_string()), async {
            assert_eq!(current_session_id(), Some("test-session-123".to_string()));
        })
        .await;
    }

    #[tokio::test]
    async fn test_session_id_none_when_not_set() {
        let id = current_session_id();
        assert_eq!(id, None);
    }

    #[tokio::test]
    async fn test_session_id_none_when_explicitly_none() {
        with_session_id(None, async {
            assert_eq!(current_session_id(), None);
        })
        .await;
    }

    #[tokio::test]
    async fn test_session_id_scoped_correctly() {
        assert_eq!(current_session_id(), None);

        with_session_id(Some("outer-session".to_string()), async {
            assert_eq!(current_session_id(), Some("outer-session".to_string()));

            with_session_id(Some("inner-session".to_string()), async {
                assert_eq!(current_session_id(), Some("inner-session".to_string()));
            })
            .await;

            assert_eq!(current_session_id(), Some("outer-session".to_string()));
        })
        .await;

        assert_eq!(current_session_id(), None);
    }

    #[tokio::test]
    async fn test_session_id_across_await_points() {
        with_session_id(Some("persistent-session".to_string()), async {
            assert_eq!(current_session_id(), Some("persistent-session".to_string()));

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            assert_eq!(current_session_id(), Some("persistent-session".to_string()));
        })
        .await;
    }

    #[tokio::test]
    async fn test_action_scope_sets_session_context() {
        let scope = ActionRequiredScope {
            session_id: Some("session-1".to_string()),
            thread_id: Some("thread-1".to_string()),
            turn_id: Some("turn-1".to_string()),
        };

        with_action_scope(scope.clone(), async {
            assert_eq!(current_session_id(), Some("session-1".to_string()));
            assert_eq!(current_action_scope(), Some(scope));
        })
        .await;
    }
}
