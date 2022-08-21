use iced::scrollable;
use iced::text_input;
use tokio::sync::RwLock;

use crate::app::Distrox;
use crate::app::Message;
use crate::timeline::Timeline;

pub(super) fn handle_message(app: &mut Distrox, message: Message) -> iced::Command<Message> {
    log::trace!("Received message: {}", message.description());

    if let Distrox::Loading {
        gossip_subscription_recv,
    } = app
    {
        if let Message::Loaded(profile) = message {
            *app = Distrox::Loaded {
                profile,

                // Don't even try to think what hoops I am jumping through here...
                gossip_subscription_recv: std::mem::replace(
                    gossip_subscription_recv,
                    RwLock::new(tokio::sync::oneshot::channel().1),
                ),
                scroll: scrollable::State::default(),
                input: text_input::State::default(),
                input_value: String::default(),
                timeline: Timeline::new(),
                log_visible: false,
                log: std::collections::VecDeque::with_capacity(1000),
            };
            return iced::Command::none();
        }
    }

    match (app, message) {
        (Distrox::Loading { .. }, _) => iced::Command::none(),

        (Distrox::Loaded { input_value, .. }, Message::InputChanged(input)) => {
            *input_value = input;
            iced::Command::none()
        }

        (
            Distrox::Loaded {
                profile,
                input_value,
                ..
            },
            Message::CreatePost,
        ) => {
            if !input_value.is_empty() {
                let input = input_value.clone();
                let profile = profile.clone();
                log::trace!("Posting...");
                iced::Command::perform(
                    async move {
                        log::trace!("Posting: '{}'", input);
                        profile.write().await.post_text(input).await
                    },
                    |res| match res {
                        Ok(cid) => Message::PostCreated(cid),
                        Err(e) => Message::PostCreationFailed(e.to_string()),
                    },
                )
            } else {
                iced::Command::none()
            }
        }

        (
            Distrox::Loaded {
                profile,
                input_value,
                ..
            },
            Message::PostCreated(cid),
        ) => {
            *input_value = String::new();
            log::info!("Post created: {}", cid);

            let profile = profile.clone();
            iced::Command::perform(
                async move {
                    if let Err(e) = profile.read().await.save().await {
                        Message::ProfileStateSavingFailed(e.to_string())
                    } else {
                        Message::ProfileStateSaved
                    }
                },
                |m: Message| -> Message { m },
            )
        }

        (_, Message::ProfileStateSaved) => {
            log::info!("Profile state saved");
            iced::Command::none()
        }

        (_, Message::ProfileStateSavingFailed(e)) => {
            log::error!("Saving profile failed: {}", e);
            iced::Command::none()
        }

        (_, Message::PostCreationFailed(err)) => {
            log::error!("Post creation failed: {}", err);
            iced::Command::none()
        }

        (Distrox::Loaded { timeline, .. }, Message::PostLoaded((payload, content))) => {
            timeline.push(payload, content);
            iced::Command::none()
        }

        (_, Message::PostLoadingFailed) => {
            log::error!("Failed to load some post, TODO: Better error logging");
            iced::Command::none()
        }

        (_, Message::TimelineScrolled(f)) => {
            log::trace!("Timeline scrolled: {}", f);
            iced::Command::none()
        }

        (Distrox::Loaded { log_visible, .. }, Message::ToggleLog) => {
            log::trace!("Log toggled");
            *log_visible = !*log_visible;
            iced::Command::none()
        }

        (_, Message::GossipMessage(source, msg)) => {
            log::trace!("Received Gossip from {}: {:?}", source, msg);
            iced::Command::perform(
                async { Message::GossipHandled(msg) },
                |m: Message| -> Message { m },
            )
        }

        (Distrox::Loaded { log, .. }, Message::GossipHandled(msg)) => {
            use distrox_lib::gossip::GossipMessage;

            log::trace!("Gossip handled, adding to log: {:?}", msg);
            let msg = match msg {
                GossipMessage::CurrentProfileState { peer_id, cid } => {
                    format!("Peer {:?} is at {:?}", peer_id, cid)
                }
            };
            log.push_back(msg);
            while log.len() > 1000 {
                let _ = log.pop_front();
            }
            iced::Command::none()
        }

        (Distrox::Loaded { profile, .. }, Message::PublishGossipAboutMe) => {
            let profile = profile.clone();
            iced::Command::perform(
                async move {
                    if let Err(e) = profile
                        .read()
                        .await
                        .gossip_own_state("distrox".to_string())
                        .await
                    {
                        Message::GossippingFailed(e.to_string())
                    } else {
                        Message::OwnStateGossipped
                    }
                },
                |m: Message| -> Message { m },
            )
        }

        (Distrox::Loaded { log, .. }, Message::OwnStateGossipped) => {
            log::trace!("Gossipped own state");
            log.push_back("Gossipped own state".to_string());
            iced::Command::none()
        }

        (Distrox::Loaded { log, .. }, Message::GossippingFailed(e)) => {
            log::trace!("Gossipped failed: {}", e);
            log.push_back(format!("Gossipped failed: {}", e));
            iced::Command::none()
        }

        (Distrox::Loaded { .. }, msg) => {
            log::warn!("Unhandled message: {:?}", msg);
            iced::Command::none()
        }
    }
}
