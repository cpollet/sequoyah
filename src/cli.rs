use clap::Parser;
use std::collections::HashSet;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Opts {
    /// The source file. If not provided the source is read from <STDIN>.
    #[clap(short, long)]
    input: Option<String>,

    /// The destination file. If not provided the source is written to <STDOUT>.
    #[clap(short, long)]
    output: Option<String>,

    /// Overlays to show.
    #[clap(short = 'l', long)]
    overlays: Vec<String>,

    /// Show all overlays
    #[clap(short, long, conflicts_with = "overlays")]
    all_overlays: bool,
}

impl Opts {
    pub fn input(&self) -> Option<&str> {
        self.input.as_deref()
    }

    pub fn output(&self) -> Option<&str> {
        self.output.as_deref()
    }

    pub fn overlays(&self) -> HashSet<&str> {
        if self.all_overlays {
            ["*"].into_iter().collect()
        } else {
            self.overlays.iter().map(AsRef::as_ref).collect()
        }
    }
}
