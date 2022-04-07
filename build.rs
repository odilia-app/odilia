use std::{
    env,
    fs::File,
    io::{self, prelude::*},
    path::PathBuf,
};

use clap::CommandFactory;
use clap_complete::Shell;

#[allow(dead_code)]
#[path = "src/args.rs"]
mod args;

fn main() -> io::Result<()> {
    let out_dir = match env::var_os("OUT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => return Ok(()),
    };
    let mut completions_rs = File::create(out_dir.join("completions.rs"))?;

    let shells: &[Shell] = &[
        #[cfg(feature = "bash_completion")]
        Shell::Bash,
        #[cfg(feature = "elvish_completion")]
        Shell::Elvish,
        #[cfg(feature = "fish_completion")]
        Shell::Fish,
        #[cfg(feature = "powershell_completion")]
        Shell::PowerShell,
        #[cfg(feature = "zsh_completion")]
        Shell::Zsh,
    ];

    let mut cmd = args::Args::command();
    for &shell in shells {
        let path = clap_complete::generate_to(shell, &mut cmd, env!("CARGO_PKG_NAME"), &out_dir)?;
        let shell_str = shell.to_string().to_uppercase();
        writeln!(
            completions_rs,
            "pub const {}_COMPLETIONS: &'static str = include_str!({:?});",
            shell_str, path
        )?;
    }

    Ok(())
}
