use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
};

use crate::data_type::DataType;

use super::super::_base::Provider;

fn get_manager() -> Result<GlobalSystemMediaTransportControlsSessionManager, ()> {
    return GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|manager| manager.get())
        .map_err(|e| tracing::error!("Can not get Session Manager: {}", e));
}

struct MediaState {
    sent_title: String,
}

const MEDIA_ARTIST_MAX_CHARS: usize = 4;
const MEDIA_TITLE_MAX_CHARS: usize = 4;

fn sanitize_media_text(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c == '\r' || c == '\n' || c == '\t' {
                ' '
            } else if c.is_ascii() {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn status_prefix(session: &GlobalSystemMediaTransportControlsSession) -> &'static str {
    let status = session
        .GetPlaybackInfo()
        .and_then(|info| info.PlaybackStatus())
        .unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed);

    if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
        ">"
    } else if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused {
        "|"
    } else {
        " "
    }
}

fn status_prefix_byte(session: &GlobalSystemMediaTransportControlsSession) -> u8 {
    status_prefix(session).as_bytes()[0]
}

fn send_media_title_with_lead(
    lead_byte: u8,
    payload: &str,
    data_sender: &broadcast::Sender<Vec<u8>>,
) {
    let mut data = payload.as_bytes().to_vec();
    data.truncate(10);
    data.insert(0, lead_byte);
    data.insert(0, DataType::MediaTitle as u8);
    if let Err(e) = data_sender.send(data) {
        tracing::error!("Can not send media title data: {}", e);
    }

    std::thread::sleep(std::time::Duration::from_millis(10));
}

fn sync_and_send(session: &GlobalSystemMediaTransportControlsSession, state: &Mutex<MediaState>, data_sender: &broadcast::Sender<Vec<u8>>) {
    if let Some((artist, title)) = get_media_data(session) {
        let clean_artist = sanitize_media_text(&artist);
        let clean_title = sanitize_media_text(&title);
        let artist_line: String = clean_artist.chars().take(MEDIA_ARTIST_MAX_CHARS).collect();
        let title_line: String = clean_title.chars().take(MEDIA_TITLE_MAX_CHARS).collect();
        let top_line = if artist_line.is_empty() {
            "----".to_string()
        } else {
            format!("{:<4}", artist_line)
        };
        let bottom_line = if title_line.is_empty() {
            "-".to_string()
        } else {
            title_line
        };
        let display_title_payload = format!("{}\n{:<4}", top_line, bottom_line);
        let lead = status_prefix_byte(session);
        let display_title_state = format!("{}{}", lead as char, display_title_payload);
        let mut state = state.lock().unwrap();

        if state.sent_title != display_title_state {
            send_media_title_with_lead(lead, &display_title_payload, data_sender);
            state.sent_title = display_title_state;
        }
    }
}

fn handle_session(
    session: &GlobalSystemMediaTransportControlsSession,
    state: &Arc<Mutex<MediaState>>,
    data_sender: &broadcast::Sender<Vec<u8>>,
) -> Option<EventRegistrationToken> {
    sync_and_send(session, state, data_sender);

    let data_sender_playback = data_sender.clone();
    let state_playback = state.clone();
    let playback_handler = &TypedEventHandler::new(move |_session: &Option<GlobalSystemMediaTransportControlsSession>, _| {
        if let Some(session) = _session.as_ref() {
            sync_and_send(session, &state_playback, &data_sender_playback);
        }
        Ok(())
    });
    let _ = session
        .PlaybackInfoChanged(playback_handler)
        .map_err(|e| tracing::error!("Can not register PlaybackInfoChanged callback: {}", e));

    let data_sender = data_sender.clone();
    let state = state.clone();
    let session_handler = &TypedEventHandler::new(move |_session: &Option<GlobalSystemMediaTransportControlsSession>, _| {
        if let Some(session) = _session.as_ref() {
            sync_and_send(session, &state, &data_sender);
        }

        Ok(())
    });

    return session
        .MediaPropertiesChanged(session_handler)
        .map_err(|e| tracing::error!("Can not register MediaPropertiesChanged callback: {}", e))
        .ok();
}

fn get_media_data(session: &GlobalSystemMediaTransportControlsSession) -> Option<(String, String)> {
    if let Ok(media_properties) = session
        .TryGetMediaPropertiesAsync()
        .and_then(|x| x.get())
        .map_err(|e| tracing::error!("Can not get media properties: {}", e))
    {
        let artist = media_properties.Artist().unwrap_or_default().to_string();
        let title = media_properties.Title().unwrap_or_default().to_string();

        if !artist.is_empty() || !title.is_empty() {
            return Some((artist, title));
        }
    }

    None
}

pub struct MediaProvider {
    data_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
}

impl MediaProvider {
    pub fn new(data_sender: broadcast::Sender<Vec<u8>>) -> Box<dyn Provider> {
        let provider = MediaProvider {
            data_sender,
            is_started: Arc::new(AtomicBool::new(false)),
        };
        return Box::new(provider);
    }
}

impl Provider for MediaProvider {
    fn start(&self) {
        tracing::info!("Media Provider started");
        self.is_started.store(true, Relaxed);
        let data_sender = self.data_sender.clone();
        let is_started = self.is_started.clone();
        std::thread::spawn(move || {
            let mut session_token: Option<EventRegistrationToken> = None;

            if let Ok(manager) = get_manager() {
                let state = Arc::new(Mutex::new(MediaState {
                    sent_title: String::new(),
                }));

                if let Some(session) = manager.GetCurrentSession().ok() {
                    session_token = handle_session(&session, &state, &data_sender);
                }

                let handler_state = state.clone();
                let handler_data_sender = data_sender.clone();
                let handler = TypedEventHandler::new(move |_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>, _| {
                    if let Some(session) = _manager.as_ref().unwrap().GetCurrentSession().ok() {
                        if let Some(token) = session_token {
                            let _ = session.RemoveMediaPropertiesChanged(token);
                        }

                        session_token = handle_session(&session, &handler_state, &handler_data_sender);
                    }

                    Ok(())
                });

                let manager_token = manager
                    .CurrentSessionChanged(&handler)
                    .map_err(|e| tracing::error!("Can not register CurrentSessionChanged callback: {}", e));

                // Poll as a robustness fallback: WinRT's PlaybackInfoChanged event does not
                // fire reliably for every play/pause transition on every media app, which was
                // causing the on-screen status prefix to get stuck (e.g. stuck on "Playing").
                // Re-sync the current session's status/title once a second regardless of events.
                let mut tick: u32 = 0;
                loop {
                    if !is_started.load(Relaxed) {
                        break;
                    }

                    if tick % 10 == 0 {
                        if let Some(session) = manager.GetCurrentSession().ok() {
                            sync_and_send(&session, &state, &data_sender);
                        }
                    }
                    tick = tick.wrapping_add(1);

                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                if let Ok(token) = manager_token {
                    let _ = manager.RemoveCurrentSessionChanged(token);
                }

                tracing::info!("Media Provider stopped");
            }
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
