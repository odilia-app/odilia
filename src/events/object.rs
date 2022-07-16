use atspi::events::Event;
use crate::state::ScreenReaderState;

pub async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
    // Dispatch based on member
    if let Some(member) = event.member() {
    match member.as_str() {
        "StateChanged" => state_changed::dispatch(state, event).await?,
        "TextCaretMoved"=>text_caret_moved::dispatch(state, event).await?,
            member => tracing::debug!(member, "Ignoring event with unknown member"),
    }
    }
Ok(())
}

mod state_changed {
use atspi::events::Event;
use crate::state::ScreenReaderState;

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
    let path = if let Some(path) = event.path() { path } else {return Ok(()); };
    let sender = if let Some(sender) = event.sender()? { sender } else { return Ok(()); };
    let accessible = state.accessible(sender, path).await?;

    let name_fut = accessible.name();
    let description_fut = accessible.description();
    let role_fut = accessible.get_localized_role_name();
    let (name, description, role) = tokio::try_join!(name_fut, description_fut, role_fut)?;

    state.speaker.say(speech_dispatcher::Priority::Text, format!("{name}, {role}. {description}"));
    Ok(())
}
}
mod text_caret_moved {
    use crate::state::ScreenReaderState;
    use atspi::events::Event;

    pub async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
        let current_caret_position = event.detail1();
        state.speaker.say(
            speech_dispatcher::Priority::Text,
            format!("{}", current_caret_position),
        );
        Ok(())
    }
}
