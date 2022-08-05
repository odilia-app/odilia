use odilia_common::{
    input::{Key, KeyBinding, Modifiers},
    elements::ElementType,
    events::{
        ScreenReaderEventType,
    },
};
use std::collections::HashMap;
use odilia_input::events::{
    create_keybind_channel,
    

#[tokio::main]
async fn main() {
    
    let mut rx = create_keybind_channel();
    while let Some(kb) = rx.recv().await {
        println!("{:?}", kb);
    }
}
