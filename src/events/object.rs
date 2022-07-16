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

mod state_changed {
    use crate::state::ScreenReaderState;
    use atspi::events::Event;

    pub async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
        // Dispatch based on kind
        match event.kind() {
            "focused" => focused(state, event).await?,
            kind => tracing::debug!(kind, "Ignoring event with unknown kind"),
        }
        Ok(())
    }

    async fn focused(state: &ScreenReaderState, event: Event) -> zbus::Result<()> {
        // Speak the newly focused object
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
        let accessible = state.accessible(sender, path).await?;

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
mod text_caret_moved {
    use crate::state::ScreenReaderState;
    use atspi::{events::Event, text::TextProxy};
    use std::sync::atomic::{AtomicI32, Ordering};

    pub async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
        let current_caret_position = event.detail1();
        let previous_caret_position = state.previous_caret_position.load(Ordering::Relaxed);
        //this is not a typo, it's here to prevent an edge case from triggering, therefore preemptively squashing a potential bug 
        let previous_caret_position = if previous_caret_position>0{
            previous_caret_position
        }
        else{
            current_caret_position-1
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
        let current_text_object = TextProxy::builder(state.atspi.connection())
            .destination(sender)?
            .path(path)?
            .build()
            .await?;
        let text = current_text_object
            .get_text(previous_caret_position, current_caret_position)
            .await?;
        let text = text.trim();
        state
            .previous_caret_position
            .store(current_caret_position, Ordering::Relaxed);
        state.speaker.say(speech_dispatcher::Priority::Text, text);

        Ok(())
    }
}
