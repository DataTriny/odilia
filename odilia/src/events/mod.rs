mod object;
mod document;

use std::{collections::HashMap, rc::Rc};

use futures::stream::StreamExt;
use speech_dispatcher::Priority;
use tokio::sync::mpsc::{Receiver, Sender};

use atspi::{
    accessible::Role,
    accessible_ext::{AccessibleExt, MatcherArgs},
    collection::MatchType,
    component::ScrollType,
    convertable::Convertable,
    events::Event,
};
use crate::state::ScreenReaderState;
use odilia_common::{
    events::{Direction, ScreenReaderEvent},
    modes::ScreenReaderMode,
};
use zbus_names::UniqueName;

pub async fn structural_navigation(state: &ScreenReaderState, dir: Direction, role: Role) -> zbus::Result<()> {
    let curr = match state.history_item(0).await? {
      Some(acc) => acc,
      None => return Ok(())
    };
    let roles = vec![role];
    let attributes = HashMap::new();
    let interfaces = Vec::new();
    let mt: MatcherArgs = (
        roles,
        MatchType::Invalid,
        attributes,
        MatchType::Invalid,
        interfaces,
        MatchType::Invalid,
    );
    if let Some(next) = curr.get_next(&mt, dir == Direction::Backward).await? {
        let comp = next.to_component().await?;
        let texti = next.to_text().await?;
        let focused = comp.grab_focus().await?;
        comp.scroll_to(ScrollType::TopLeft).await?;
        let caret_offset = texti.set_caret_offset(0).await?;
        tracing::debug!("Focused: {}", focused);
        tracing::debug!("Caret offset: {}", caret_offset);
        state.update_accessible(UniqueName::try_from(next.destination().as_str())?, next.path().to_owned()).await;
        let role = next.get_role().await?;
        let len = texti.character_count().await?;
        let text = texti.get_text(0, len).await?;
        state.say(Priority::Text, format!("{}, {}", text, role)).await;
    } else {
        state.say(Priority::Text, format!("No more {}s", role)).await;
    }
    Ok(())
}

pub async fn sr_event(
    state: Rc<ScreenReaderState>,
    sr_events: &mut Receiver<ScreenReaderEvent>,
) -> zbus::Result<()> {
    loop {
        let sr_event = match sr_events.recv().await {
            Some(e) => e,
            _ => continue
        };
        tracing::debug!("SR Event received");
        match sr_event {
            ScreenReaderEvent::StructuralNavigation(dir, role) => {
                 if let Err(e) = structural_navigation(&state, dir, role).await {
                    tracing::debug!(error = %e, "There was an error with the structural navigation call.");
                }
            },
            ScreenReaderEvent::StopSpeech => tracing::trace!("Stopping speech!"),
            ScreenReaderEvent::ChangeMode(ScreenReaderMode { name }) => {
                tracing::debug!("Changing mode to {:?}", name);
                //let _ = mode_channel.send(ScreenReaderMode { name }).await;
            }
            _ => {}
        };
    }
}

//#[tracing::instrument(level = "debug"i, skip(state))]
pub async fn receive(state: Rc<ScreenReaderState>, tx: Sender<Event>) {
    let events = state.atspi.event_stream();
    pin_utils::pin_mut!(events);
    loop {
        match events.next().await {
            Some(Ok(event)) => {
              if let Err(e) = tx.send(event).await {
                tracing::error!(error = %e, "Error sending atspi event");
              }
            },
            _ => tracing::debug!("Event is none"),
        }
    }
}

//#[tracing::instrument(level = "debug")]
pub async fn process(state: Rc<ScreenReaderState>, rx: &mut Receiver<Event>) {
  while let Some(event) = rx.recv().await {
    if let Err(e) = dispatch(&state, event).await {
        tracing::error!(error = %e, "Could not handle event");
    } else {
        tracing::debug!("Event handled without error");
    }
  }
}

async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
    // Dispatch based on interface
    if let Some(interface) = event.interface() {
        match interface
            .rsplit('.')
            .next()
            .expect("Interface name should contain '.'")
        {
            "Object" => object::dispatch(&state, event).await?,
            "Document" => document::dispatch(&state, event).await?,
            interface => tracing::debug!(interface, "Ignoring event with unknown interface"),
        }
    }
    Ok(())
}
