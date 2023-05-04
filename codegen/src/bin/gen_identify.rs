use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    vec,
};

use odilia_codegen::*;
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

pub fn iface_name(interface: &Interface) -> String {
	interface
		.name()
		.split('.')
		.next_back()
		.expect("An interface must have a period in the name")
		.to_string()
}

pub fn generate_wai_flag(iface: &Interface, signal: &Signal) -> String {
	format!("\t{}-{},", iface_name(iface).to_case(Case::Kebab), signal.name().to_case(Case::Kebab))
}

fn for_signals<F>(node: &Node, func: F) -> String 
	where F: Fn(&Interface, &Signal) -> String {
	node.interfaces()
		.iter()
		.map(|iface| iface.signals()
			.iter()
			.map(|signal| func(iface, signal))
			.collect::<Vec<String>>()
			.join("\n")
		)
		.collect::<Vec<String>>()
		.join("\n")
}

pub fn  get_root_node_from_xml(file_name: &str) -> Node {
    let xml_file = std::fs::File::open(file_name).expect("Cannot read file");
    Node::from_reader(&xml_file).expect("Cannot deserialize file")
}

pub fn for_interfaces<F>(node: &Node, func: F) -> String 
	where F: Fn(&Interface) -> String {
	node.interfaces()
		.iter()
		.map(|iface| func(iface))
		.collect::<Vec<String>>()
		.join("\n")
}
pub fn event_type_flags(nodes: Vec<Node>) -> String {
		let flags = nodes.iter()
			.map(|node| for_signals(node, generate_wai_flag))
			.collect::<Vec<String>>()
			.join("\n");
	format!(
r#"flags event-type {{
{flags}
}}"#)
}

pub fn main() {
    let mut generated = String::new();
		let nodes = vec![
			get_root_node_from_xml("/home/tait/Documents/atspi/xml/Event.xml"),
			get_root_node_from_xml("/home/tait/Documents/atspi/xml/Cache.xml"),
			get_root_node_from_xml("/home/tait/Documents/atspi/xml/Registry.xml"),
			get_root_node_from_xml("/home/tait/Documents/atspi/xml/Socket.xml"),
			get_root_node_from_xml("/home/tait/Documents/atspi/xml/DeviceEventListener.xml"),
		];
		println!("{}", event_type_flags(nodes));
}
