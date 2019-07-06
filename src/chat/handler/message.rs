use super::{ChatServer, ClientPacket, Id};
use crate::auth::UserInfo;
use crate::chat::{InternalId, SessionState};

use crate::error::*;
use log::*;

impl ChatServer {
    pub(super) fn handle_message(&mut self, user_id: InternalId, content: String) {
        if self.basic_check(user_id, &content).is_some() {
            let session = self
                .connections
                .get_mut(&user_id)
                .expect("could not find connection");

            if check_ratelimit(user_id, session) {
                return;
            }

            let info = session.user.as_ref().unwrap();
            let author_id = info.name.as_str().into();

            info!("User `{}` has written `{}`.", user_id, content);
            let client_packet = ClientPacket::Message {
                author_id,
                author_info: Some(UserInfo {
                    name: info.name.clone(),
                    uuid: info.uuid,
                }),
                content,
            };
            for session in self.connections.values() {
                if let Err(err) = session.addr.do_send(client_packet.clone()) {
                    warn!("Could not send message to client: {}", err);
                }
            }
        }
    }

    pub(super) fn handle_private_message(
        &mut self,
        user_id: InternalId,
        receiver: Id,
        content: String,
    ) {
        let sender_session = self
            .connections
            .get_mut(&user_id)
            .expect("could not find connection");

        if check_ratelimit(user_id, sender_session) {
            return;
        }

        if let Some(sender_session) = self.basic_check(user_id, &content) {
            let receiver_ids = match self.ids.get(&receiver) {
                Some(ids) => ids,
                None => {
                    debug!(
                        "User `{}` tried to write to non-existing user `{}`.",
                        user_id, receiver
                    );
                    return;
                }
            };

            for receiver_session in receiver_ids.iter().filter_map(|id| self.connections.get(id)) {
                match &receiver_session.user {
                    Some(info) if info.allow_messages => {
                        let sender_info = sender_session.user.as_ref().unwrap();
                        let author_id = sender_info.name.as_str().into();

                        let client_packet = ClientPacket::PrivateMessage {
                            author_id,
                            author_info: Some(UserInfo {
                                name: sender_info.name.clone(),
                                uuid: sender_info.uuid,
                            }),
                            content: content.clone(),
                        };
                        info!(
                            "User `{}` has written to `{}` privately.",
                            user_id, receiver
                        );
                        if let Err(err) = receiver_session.addr.do_send(client_packet) {
                            warn!("Could not send private message to client: {}", err);
                        } else {
                            return;
                        }
                    }
                    _ => {}
                }
            }
        }

        let _ = self
            .connections
            .get_mut(&user_id)
            .expect("could not find connection")
            .addr
            .do_send(ClientPacket::Error {
                message: ClientError::PrivateMessageNotAccepted,
            });
    }

    fn basic_check(&self, user_id: InternalId, content: &str) -> Option<&SessionState> {
        let session = self
            .connections
            .get(&user_id)
            .expect("could not find connection");

        if let Some(info) = &session.user {
            if let Err(err) = self.validator.validate(content) {
                info!("User `{}` tried to send invalid message: {}", user_id, err);
                if let Error::AxoChat { source } = err {
                    session
                        .addr
                        .do_send(ClientPacket::Error { message: source })
                        .ok();
                }

                return None;
            }
            if self.moderation.is_banned(&info.uuid) {
                info!("User `{}` tried to send message while banned", user_id);
                session
                    .addr
                    .do_send(ClientPacket::Error {
                        message: ClientError::Banned,
                    })
                    .ok();

                return None;
            }

            Some(session)
        } else {
            info!("`{}` is not logged in.", user_id);
            session
                .addr
                .do_send(ClientPacket::Error {
                    message: ClientError::NotLoggedIn,
                })
                .ok();
            None
        }
    }
}

fn check_ratelimit(user_id: InternalId, session: &mut SessionState) -> bool {
    if session.rate_limiter.check_new_message() {
        info!(
            "User `{}` tried to send message, but was rate limited.",
            user_id
        );
        session
            .addr
            .do_send(ClientPacket::Error {
                message: ClientError::RateLimited,
            })
            .ok();
        true
    } else {
        false
    }
}
