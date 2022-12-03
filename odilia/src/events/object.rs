use crate::state::ScreenReaderState;
use atspi::events::Event;

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
    use crate::state::ScreenReaderState;
    use atspi::{accessible, convertable::Convertable, events::Event};
    use ssip_client::Priority;

    // TODO: left/right vs. up/down, and use generated speech
    pub async fn text_cursor_moved(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
        let current_caret_pos = event.detail1();
        let previous_caret_pos = state.previous_caret_position.get();
        state.previous_caret_position.set(current_caret_pos);
        let (_start, _end) = match current_caret_pos > previous_caret_pos {
            true => (previous_caret_pos, current_caret_pos),
            false => (current_caret_pos, previous_caret_pos),
        };
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
        let conn = state.connection().clone();
        let accessible = accessible::new(&conn, sender.clone(), path.clone()).await?;
        let _last_accessible = match state.history_item(0).await? {
            Some(acc) => acc,
            None => return Ok(()),
        };
        let last_last_accessible = match state.history_item(1).await? {
            Some(acc) => acc,
            None => return Ok(()),
        };
        state.update_accessible(sender, path).await;

        // in the case that this is not a tab navigation
        // TODO: algorithm that only triggers this when a tab navigation is known to have not occured. How the fuck am I supposed to know how that works?
        // Ok, start out with the basics: if a focus event has recently occuredm, there is a good chance that this function is about to get triggered as well. So, for one, a tab navigation GUARENTEES that the last_accessible will be equal to the curent accessible.
        if accessible == last_last_accessible {
            let txt = accessible.to_text().await?;
            let len = txt.character_count().await?;
            // TODO: improve text readout
            state
                .say(Priority::Text, format!("{}", txt.get_text(0, len).await?))
                .await;
        }
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
    use atspi::{accessible, events::Event};

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

        let (name, description, role, relation) = tokio::try_join!(
        accessible.name(),
        accessible.description(),
        accessible.get_localized_role_name(),
        accessible.get_relation_set(),
        )?;
        tracing::debug!("Focus event received on: {} with role {}", path, role);
        tracing::debug!("Relations: {:?}", relation);

        state
            .say(ssip_client::Priority::Text, format!("{name}, {role}. {description}"))
            .await;

        Ok(())
    }
}
