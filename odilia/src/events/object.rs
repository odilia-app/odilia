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
    use std::cmp::{max, min};

    use atspi::{accessible, events::Event, text};
    use crate::state::ScreenReaderState;

    pub async fn text_cursor_moved(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
        let current_caret_pos = event.detail1();
        let path = if let Some(path) = event.path() {
            path
        } else {
            return Ok(());
        };
        let sender = if let Some(sender) = event.sender()? {
            sender
        } else {
            return Ok(());
        };
        let conn = state.connection();
        let text = text::new(&conn.clone(), sender.to_owned(), path.to_owned()).await?;
        let accessible = accessible::new(&conn, sender.clone(), path.clone()).await?;
        if let Some(latest) = state.history_item(0).await? {
          if let Some(second) = state.history_item(1).await? {
          if latest == accessible && second != accessible && current_caret_pos == 0 {
              tracing::trace!("Caret is moved to latest accessible and the second latest isn't the same and te cursor is at zero; this is usually a result of a structural navigation or tab. Do not read out.");
              state.update_accessible(sender, path).await;
              return Ok(());
            }
          }
        }
        let line = text.get_string_at_offset(current_caret_pos, text::TextGranularity::Line).await?.0;
        state.update_accessible(sender, path).await;

        // TODO this just won't work on the first two accessibles we call. oh well.
        let latest_accessible = match state.history_item(0).await? {
          Some(acc) => acc,
          None => return Ok(()),
        };
        let second_latest_accessible = match state.history_item(0).await? {
          Some(acc) => acc,
          None => return Ok(())
        };
        // if this is the same accessible as previously acted upon, and caret position is 0
        // This will be true if the user has just tabbed into a new accessible.
        if latest_accessible.path() == accessible.path()
            && second_latest_accessible.path() != accessible.path()
            && current_caret_pos == 0
        {
            tracing::debug!("Tabbed selection detected. Do no re-speak due to caret navigation.");
        } else {
            tracing::debug!("Tabbed selection not detected.");
            tracing::debug!("Speaking normal caret navigation");
            if !line.is_empty() {
              state.say(speech_dispatcher::Priority::Text, format!("{}", line)).await;
            }
        }

        // update caret position
        state.previous_caret_position.set(current_caret_pos);
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
    use atspi::{accessible, events::Event};
    use crate::state::ScreenReaderState;

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
        let path = if let Some(path) = event.path() {
            path.to_owned()
        } else {
            return Ok(());
        };
        let sender = if let Some(sender) = event.sender()? {
            sender.to_owned()
        } else {
            return Ok(());
        };
        let conn = state.connection();
        let accessible = accessible::new(&conn.clone(), sender.clone(), path.clone()).await?;
        if let Some(curr) = state.history_item(0).await? {
          if curr == accessible {
            return Ok(());
          }
        }
        state.update_accessible(sender.to_owned(), path.to_owned()).await;

        let name = accessible.name().await?;
        let description = accessible.description().await?;
        let role = accessible.get_localized_role_name().await?;
        tracing::debug!("Focus event received on: {} with role {}", path, role);
        let relation = accessible.get_relation_set().await?;
        tracing::debug!("Relations: {:?}", relation);

        state.say(
            speech_dispatcher::Priority::Text,
            format!("{name}, {role}. {description}"),
        )
        .await;

        Ok(())
    }
}
