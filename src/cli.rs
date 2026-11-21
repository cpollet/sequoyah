use clap::Parser;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Opts {
    /// The source file. If not provided the source is read from <STDIN>.
    #[clap(short, long)]
    input: Option<String>,

    /// The destination file. If set to '-' or if unset and reading from <STDIN>, the output is
    /// written to <STDOUT>.
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

    pub fn output(&self) -> OutputFile<'_> {
        match self.output.as_deref() {
            Some("-") => OutputFile::Stdout,
            Some(filename) => OutputFile::File(Cow::Borrowed(filename)),
            None => match &self.input {
                Some(input) => {
                    let output = PathBuf::from_str(input.as_str())
                        .unwrap()
                        .with_extension("puml");

                    OutputFile::File(Cow::Owned(
                        output.to_str().expect("was read from a string").to_owned(),
                    ))
                }
                None => OutputFile::Stdout,
            },
        }
    }

    pub fn overlays(&self) -> HashSet<&str> {
        if self.all_overlays {
            ["*"].into_iter().collect()
        } else {
            self.overlays.iter().map(AsRef::as_ref).collect()
        }
    }
}

pub enum OutputFile<'opt> {
    Stdout,
    File(Cow<'opt, str>),
}
