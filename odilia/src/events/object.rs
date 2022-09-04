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
        let last_caret_pos = state.previous_caret_position.get();
        let current_caret_pos = event.detail1();

        let start = min(last_caret_pos, current_caret_pos);
        let end = max(last_caret_pos, current_caret_pos);

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
        let name = text.get_text(start, end).await?;
        state.update_accessible(sender, path).await;

        // this just won't work on the first two accessibles we call. oh well.
        let latest_accessible = state.history_item(0).await?;
        let second_latest_accessible = state.history_item(0).await?;
        // if this is the same accessible as previously acted upon, and caret position is 0
        // This will be true if the user has just tabbed into a new accessible.
        if latest_accessible.path() == accessible.path()
            && second_latest_accessible.path() != accessible.path()
            && current_caret_pos == 0
        {
            tracing::debug!("Tabbed selection detected. Do no re-speak due to caret navigation.");
        } else {
            tracing::debug!("Tabbed selection not detected.");
            if !name.is_empty() {
                tracing::debug!("Speaking normal caret navigation");
                state.say(speech_dispatcher::Priority::Text, name).await;
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

        state.update_accessible(sender.to_owned(), path.to_owned()).await;

        let name = accessible.name().await?;
        let description = accessible.description().await?;
        let role = accessible.get_localized_role_name().await?;
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
