use atspi::events::Event;
use crate::state::ScreenReaderState;

pub async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
    // Dispatch based on member
    if let Some(member) = event.member() {
    match member.as_str() {
        "StateChanged" => state_changed::dispatch(state, event).await?,
        "TextCaretMoved" => text_caret_moved::dispatch(state, event).await?,
            member => tracing::debug!(member, "Ignoring event with unknown member"),
        }
    }
    Ok(())
}

mod text_caret_moved {
use atspi::{
  events::Event,
};
use crate::state::ScreenReaderState;
use std::cmp::{
  min,
  max
};
use std::sync::{
    atomic::Ordering,
};

pub async fn text_cursor_moved(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
  let last_caret_pos = state.previous_caret_position.load(Ordering::Relaxed);
  let current_caret_pos = event.detail1();

  let start = min(last_caret_pos, current_caret_pos);
  let end = max(last_caret_pos, current_caret_pos);

  let path = if let Some(path) = event.path() { path } else {return Ok(()); };
  let sender = if let Some(sender) = event.sender()? { sender } else { return Ok(()); };
  let text = state.text(sender.to_owned(), path.to_owned()).await?;
  let accessible = state.accessible(sender, path).await?;
  let name = text.get_text(start, end).await?;
  
  let accessible_history_m = std::sync::Arc::clone(&state.accessible_history);
  let accessible_history_q = accessible_history_m.lock().await;
  let mut accessible_history = accessible_history_q.iter();
  // this just won't work on the first two accessibles we call. oh well.
  if let Some(latest_accessible_parts) = accessible_history.next() {
      if let Some(second_latest_accessible_parts) = accessible_history.next() {
          let (latest_sender,latest_path) = latest_accessible_parts;
          let (second_latest_sender, second_latest_path) = second_latest_accessible_parts;
          let latest_accessible = state.accessible(latest_sender.to_owned(), latest_path.to_owned()).await?;
          let second_latest_accessible = state.accessible(second_latest_sender.to_owned(), second_latest_path.to_owned()).await?;
          // if this is the same accessible as previously acted upon, and caret position is 0
          // This will be true if the user has just tabbed into a new accessible.
          if latest_accessible.path() == accessible.path() &&
             second_latest_accessible.path() != accessible.path() &&
             current_caret_pos == 0 {
              tracing::debug!("Tabbed selection detected. Do no re-speak due to caret navigation.");
          } else {
              tracing::debug!("Tabbed selection not detected.");
              if name.len() > 0 {
                tracing::debug!("Speaking normal caret navigation");
                state.speaker.say(speech_dispatcher::Priority::Text, format!("{name}"));
              }
          }
      }
  }

  // update caret position
  state.previous_caret_position.store(current_caret_pos, Ordering::Relaxed);
  Ok(())
}

pub async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
  // Dispatch based on kind
  match event.kind() {
    "" => text_cursor_moved(state, event).await?,
    kind => tracing::debug!(kind, "Ignoring event with unknown kind"),
  }
  Ok(())
}

} // end of text_caret_moved

mod state_changed {
    use crate::state::ScreenReaderState;
    use atspi::{
      events::Event,
    };

    pub async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
        // Dispatch based on kind
        match event.kind() {
            "focused" => focused(state, event).await?,
            kind => tracing::debug!(kind, "Ignoring event with unknown kind"),
        }
        Ok(())
    }

pub async fn focused(state: &ScreenReaderState, event: Event) -> zbus::Result<()> {
    // Speak the newly focused object
    let path = if let Some(path) = event.path() { path.to_owned() } else {return Ok(()); };
    let sender = if let Some(sender) = event.sender()? { sender.to_owned() } else { return Ok(()); };
    let accessible = state.accessible(sender.clone(), path.clone()).await?;

    let accessible_history_arc = std::sync::Arc::clone(&state.accessible_history);
    let mut accessible_history = accessible_history_arc.lock().await;
    accessible_history.push((sender.to_owned(),path.to_owned()));

    let name_fut = accessible.name();
    let description_fut = accessible.description();
    let role_fut = accessible.get_localized_role_name();
    let relation_fut = accessible.get_relation_set();
    let (name, description, role, relation) = tokio::try_join!(name_fut, description_fut, role_fut, relation_fut)?;
    tracing::debug!("Relations: {:?}", relation);

    state.speaker.say(
        speech_dispatcher::Priority::Text,
        format!("{name}, {role}. {description}"),
    );

    Ok(())
}
}
