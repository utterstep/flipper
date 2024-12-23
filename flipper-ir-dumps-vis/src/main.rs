use clap::Parser;
use color_eyre::eyre::WrapErr;

use flipper_ir_dumps::{dump::DumpFile, signal::ParsedSignal};

mod cli;
use cli::Cli;

mod plotting;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let contents = std::fs::read_to_string(cli.file).wrap_err("Failed to read file")?;
    let dump = DumpFile::try_from(contents.as_str());

    let dump = match dump {
        Ok(dump) => dump,
        Err(err) => {
            eprintln!("Failed decoding dump: {:?}", err);
            return Ok(());
        }
    };

    std::fs::create_dir_all(&cli.output_dir).wrap_err("Failed to create output directory")?;

    for signal in dump.signals() {
        plotting::plot_signal(&signal, &cli.output_dir)?;

        let parsed_signal = ParsedSignal::try_from(signal).wrap_err("Failed to parse signal")?;
        println!("parsed signal: {:#?}", parsed_signal);
    }

    Ok(())
}
