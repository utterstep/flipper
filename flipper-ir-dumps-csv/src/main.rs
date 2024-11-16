use clap::Parser;
use color_eyre::eyre::WrapErr;
use csv::WriterBuilder;

use flipper_ir_dumps::{dump::DumpFile, signal::ParsedSignal};

mod cli;
use cli::Cli;

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

    let mut writer = WriterBuilder::new()
        .flexible(true)
        .from_path(cli.output_file)
        .wrap_err("Failed to create CSV writer")?;

    for signal in dump.signals() {
        let parsed_signal = ParsedSignal::try_from(signal).wrap_err("Failed to parse signal")?;

        let mut record = vec![parsed_signal.name().to_owned()];
        record.extend(
            parsed_signal
                .packets()
                .iter()
                .map(|packet| packet.to_string()),
        );

        writer
            .write_record(record)
            .wrap_err("Failed to write record")?;
    }

    Ok(())
}
