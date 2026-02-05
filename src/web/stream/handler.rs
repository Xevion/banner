//! Stream WebSocket handler.

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tracing::debug;

use crate::events::{AuditLogEvent, DomainEvent};
use crate::state::AppState;
use crate::web::extractors::AdminUser;
use crate::web::stream::protocol::{
    STREAM_PROTOCOL_VERSION, StreamClientMessage, StreamDelta, StreamError, StreamErrorCode,
    StreamKind, StreamServerMessage, StreamSnapshot,
};
use crate::web::stream::streams::{audit_log, scrape_jobs};
use crate::web::stream::subscriptions::{Subscription, SubscriptionRegistry, build_subscription};
use crate::web::ws::ScrapeJobEvent;

/// Outcome of processing a single client WebSocket message.
enum ClientMessageResult {
    /// Message processed successfully; continue the loop.
    Continue,
    /// A protocol-level error was sent to the client; continue the loop.
    ErrorSent,
    /// A WebSocket send failed; the connection is dead.
    Disconnected,
}

impl ClientMessageResult {
    /// Convert the result of a `send_error` call (true = sent, false = send failed).
    fn from_error_send(sent: bool) -> Self {
        if sent {
            Self::ErrorSent
        } else {
            Self::Disconnected
        }
    }
}

/// WebSocket endpoint for real-time streams.
pub async fn stream_ws(
    ws: WebSocketUpgrade,
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_stream_ws(socket, state))
}

async fn send_message(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    message: &StreamServerMessage,
) -> bool {
    let Ok(json) = serde_json::to_string(message) else {
        return true;
    };
    sink.send(Message::Text(json.into())).await.is_ok()
}

async fn send_error(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    request_id: Option<String>,
    code: StreamErrorCode,
    message: &str,
) -> bool {
    let msg = StreamServerMessage::Error {
        request_id,
        code,
        message: message.to_string(),
    };
    send_message(sink, &msg).await
}

async fn handle_stream_ws(socket: WebSocket, state: AppState) {
    debug!("stream WebSocket connected");

    let (mut sink, mut stream) = socket.split();
    let ready = StreamServerMessage::Ready {
        protocol_version: STREAM_PROTOCOL_VERSION,
    };
    if !send_message(&mut sink, &ready).await {
        return;
    }

    let mut registry = SubscriptionRegistry::new();

    let (mut cursor, mut head_watch) = state.events.subscribe();

    loop {
        tokio::select! {
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if matches!(
                            handle_client_message(&mut sink, &state, &mut registry, &text).await,
                            ClientMessageResult::Disconnected
                        ) {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            result = head_watch.changed() => {
                if result.is_err() {
                    break;
                }
                // Check for lag - cursor fell behind the buffer's oldest event
                if cursor < state.events.base_offset() {
                    if !resync_all(&mut sink, &state, &mut registry).await {
                        break;
                    }
                    // Start from oldest available event to avoid missing any
                    cursor = state.events.base_offset();
                    continue;
                }
                // Process all events from cursor to head
                let mut send_failed = false;
                while let Some(event) = state.events.read(cursor) {
                    if !dispatch_event(&mut sink, &state, &mut registry, event).await {
                        send_failed = true;
                        break;
                    }
                    cursor += 1;
                }
                if send_failed {
                    break;
                }
            }
        }
    }

    debug!("stream WebSocket disconnected");
}

async fn handle_client_message(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    registry: &mut SubscriptionRegistry,
    text: &str,
) -> ClientMessageResult {
    let parsed = match serde_json::from_str::<StreamClientMessage>(text) {
        Ok(msg) => msg,
        Err(_) => {
            let sent = send_error(
                sink,
                None,
                StreamErrorCode::InvalidMessage,
                "Invalid message",
            )
            .await;
            return ClientMessageResult::from_error_send(sent);
        }
    };

    match parsed {
        StreamClientMessage::Subscribe {
            request_id,
            stream,
            filter,
        } => {
            let sub_id = registry.allocate_id();

            let subscription = match build_subscription(stream, filter) {
                Ok(sub) => sub,
                Err(StreamError { code, message }) => {
                    let sent = send_error(sink, Some(request_id), code, &message).await;
                    return ClientMessageResult::from_error_send(sent);
                }
            };

            registry.insert(sub_id.clone(), subscription);
            let subscribed = StreamServerMessage::Subscribed {
                request_id,
                subscription_id: sub_id.clone(),
                stream,
            };
            if !send_message(sink, &subscribed).await {
                return ClientMessageResult::Disconnected;
            }

            if !send_snapshot(sink, state, registry, &sub_id).await {
                return ClientMessageResult::Disconnected;
            }
        }
        StreamClientMessage::Modify {
            request_id,
            subscription_id,
            filter,
        } => {
            let Some(subscription) = registry.get_mut(&subscription_id) else {
                let sent = send_error(
                    sink,
                    Some(request_id),
                    StreamErrorCode::UnknownSubscription,
                    "Unknown subscription",
                )
                .await;
                return ClientMessageResult::from_error_send(sent);
            };

            let stream = subscription.kind();
            let updated = match build_subscription(stream, filter) {
                Ok(sub) => sub,
                Err(StreamError { code, message }) => {
                    let sent = send_error(sink, Some(request_id), code, &message).await;
                    return ClientMessageResult::from_error_send(sent);
                }
            };

            *subscription = updated;
            let modified = StreamServerMessage::Modified {
                request_id,
                subscription_id: subscription_id.clone(),
            };
            if !send_message(sink, &modified).await {
                return ClientMessageResult::Disconnected;
            }

            if !send_snapshot(sink, state, registry, &subscription_id).await {
                return ClientMessageResult::Disconnected;
            }
        }
        StreamClientMessage::Unsubscribe {
            request_id,
            subscription_id,
        } => {
            registry.remove(&subscription_id);
            let msg = StreamServerMessage::Unsubscribed {
                request_id,
                subscription_id,
            };
            if !send_message(sink, &msg).await {
                return ClientMessageResult::Disconnected;
            }
        }
        StreamClientMessage::Ping {
            request_id,
            timestamp,
        } => {
            let pong = StreamServerMessage::Pong {
                request_id,
                timestamp,
            };
            if !send_message(sink, &pong).await {
                return ClientMessageResult::Disconnected;
            }
        }
    }

    ClientMessageResult::Continue
}

async fn send_snapshot(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    registry: &mut SubscriptionRegistry,
    subscription_id: &str,
) -> bool {
    let Some(subscription) = registry.get_mut(subscription_id) else {
        return true;
    };

    match subscription {
        Subscription::ScrapeJobs { filter, known_ids } => {
            let snapshot = match scrape_jobs::build_snapshot(&state.db_pool, filter).await {
                Ok(jobs) => {
                    *known_ids = jobs.iter().map(|job| job.id).collect();
                    StreamSnapshot::ScrapeJobs { jobs }
                }
                Err(_) => {
                    return send_error(
                        sink,
                        None,
                        StreamErrorCode::InternalError,
                        "Failed to load scrape jobs snapshot",
                    )
                    .await;
                }
            };

            send_message(
                sink,
                &StreamServerMessage::Snapshot {
                    subscription_id: subscription_id.to_string(),
                    snapshot,
                },
            )
            .await
        }
        Subscription::AuditLog { filter } => {
            let snapshot = match audit_log::build_snapshot(&state.db_pool, filter).await {
                Ok(entries) => StreamSnapshot::AuditLog { entries },
                Err(_) => {
                    return send_error(
                        sink,
                        None,
                        StreamErrorCode::InternalError,
                        "Failed to load audit log snapshot",
                    )
                    .await;
                }
            };

            send_message(
                sink,
                &StreamServerMessage::Snapshot {
                    subscription_id: subscription_id.to_string(),
                    snapshot,
                },
            )
            .await
        }
    }
}

async fn dispatch_event(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    registry: &mut SubscriptionRegistry,
    event: DomainEvent,
) -> bool {
    match event {
        DomainEvent::ScrapeJob(scrape_event) => {
            dispatch_scrape_job_event(sink, state, registry, scrape_event).await
        }
        DomainEvent::AuditLog(audit_event) => {
            dispatch_audit_log_event(sink, registry, audit_event).await
        }
    }
}

async fn resync_all(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    registry: &mut SubscriptionRegistry,
) -> bool {
    resync_scrape_jobs(sink, state, registry).await && resync_audit_log(sink, state, registry).await
}

async fn dispatch_scrape_job_event(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    registry: &mut SubscriptionRegistry,
    event: ScrapeJobEvent,
) -> bool {
    let mut job_details: Option<crate::web::ws::ScrapeJobDto> = None;

    for (subscription_id, subscription) in registry.iter_mut() {
        let Subscription::ScrapeJobs { filter, known_ids } = subscription else {
            continue;
        };

        let matches =
            scrape_jobs::event_matches(&state.db_pool, filter, known_ids, &event, &mut job_details)
                .await;

        if matches {
            let delta = StreamServerMessage::Delta {
                subscription_id: subscription_id.clone(),
                delta: StreamDelta::ScrapeJobs {
                    event: event.clone(),
                },
            };
            if !send_message(sink, &delta).await {
                return false;
            }
        }
    }

    true
}

async fn dispatch_audit_log_event(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    registry: &mut SubscriptionRegistry,
    event: AuditLogEvent,
) -> bool {
    for (subscription_id, subscription) in registry.iter_mut() {
        let Subscription::AuditLog { filter } = subscription else {
            continue;
        };

        let entries = audit_log::filter_entries(filter, &event.entries);
        if entries.is_empty() {
            continue;
        }

        let delta = StreamServerMessage::Delta {
            subscription_id: subscription_id.clone(),
            delta: StreamDelta::AuditLog { entries },
        };
        if !send_message(sink, &delta).await {
            return false;
        }
    }

    true
}

async fn resync_scrape_jobs(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    registry: &mut SubscriptionRegistry,
) -> bool {
    let ids = registry.ids_for_kind(StreamKind::ScrapeJobs);
    for subscription_id in ids {
        if !send_snapshot(sink, state, registry, &subscription_id).await {
            return false;
        }
    }
    true
}

async fn resync_audit_log(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    registry: &mut SubscriptionRegistry,
) -> bool {
    let ids = registry.ids_for_kind(StreamKind::AuditLog);
    for subscription_id in ids {
        if !send_snapshot(sink, state, registry, &subscription_id).await {
            return false;
        }
    }
    true
}
