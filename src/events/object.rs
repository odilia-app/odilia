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
  let test = accessible.accessible_id().await?;
  tracing::debug!("Current: {}", test);
  tracing::debug!("Empty? {}", test.is_empty());
  
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
    use zbus::{
      zvariant::ObjectPath,
      names::UniqueName,
    };
    use crate::state::ScreenReaderState;
    use atspi::{
      events::Event,
      collection::{
        MatchType,
        TreeTraversalType,
        CollectionProxy,
        SortOrder,
      },
      accessible::Role
    };
    use std::collections::HashMap;
    use crate::structural::{
      make_accessible,
      get_ancestor_with_role,
      find_with_role,
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
    /*
    let root = get_ancestor_with_role(state, &accessible, Role::Application).await?;
    let ints = root.get_interfaces().await?;
    let (rsender,rpath) = root.get_application().await?;
    let collection = state.collection(
      UniqueName::try_from(rsender)?,
      ObjectPath::try_from(rpath)?
    ).await?;
    let matcher: (
      &[i32],
      MatchType,
      HashMap<&str, &str>,
      MatchType,
      &[i32],
      MatchType,
      &[&str],
      MatchType,
      bool) = (
        &[],
        MatchType::Empty,
        HashMap::new(),
        MatchType::Empty,
        &[88],
        MatchType::All,
        &[],
        MatchType::Empty,
        false);
    let wtf = collection.get_matches_to(
      &path.clone(),
      &matcher,
      SortOrder::Flow,
      TreeTraversalType::Inorder,
      true,
      5, // always return all results; no maximum
      true // unused
    ).await?;
    tracing::debug!("Found {} results with search", wtf.len());
    for wt in wtf {
      let acc = make_accessible(state.atspi.connection(), wt).await.unwrap();
      tracing::debug!("Link is: {}", acc.name().await?);
    }
    tracing::debug!("Implements interfaces: {:?}", accessible.get_interfaces().await?);
    */
    if let Ok(next_link_op) = find_with_role(state, &accessible, Role::Link, false).await {
        if next_link_op.is_some() {
            tracing::debug!("Next link found!");
        } else {
            tracing::debug!("Did not find next link.");
        }
    }


    let accessible_history_arc = std::sync::Arc::clone(&state.accessible_history);
    let mut accessible_history = accessible_history_arc.lock().await;
    accessible_history.push((sender.to_owned(),path.to_owned()));

    let name_fut = accessible.name();
    let description_fut = accessible.description();
    let role_fut = accessible.get_localized_role_name();
    let (name, description, role) = tokio::try_join!(name_fut, description_fut, role_fut)?;

    state.speaker.say(
        speech_dispatcher::Priority::Text,
        format!("{name}, {role}. {description}"),
    );

    Ok(())
}
}
