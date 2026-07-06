use std::collections::HashMap;

use crate::cli::apply_preset::apply_preset;
use crate::cli::unapply_preset::unapply_preset;
use crate::core::config::ConfigManager;
use crate::core::{config::Config, preset::StageResult};
use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(
    name = "bakeryos-preset",
    author = "BakeryOS Team",
    version,
    about = "A CLI utility to manage and apply system configuration presets for BakeryOS.",
    long_about = "BakeryOS Preset is a powerful command-line tool designed to parse, validate, and automate the deployment of YAML-based configuration presets into the BakeryOS environment efficiently and safely."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(
        about = "Apply a preset configuration file to the system",
        long_about = "Reads, validates, and provisions all system configurations defined within the specified YAML preset file onto the current BakeryOS instance."
    )]
    Apply {
        #[arg(
            short,
            long,
            value_name = "FILE_PATH",
            help = "The relative or absolute path to the preset configuration file (.yaml)"
        )]
        path: String,
    },

    #[command(
        about = "Unapply a preset configuration file from the system",
        long_about = "Reads the specified YAML preset file and rolls back or removes all system configurations and packages defined within it from the current BakeryOS instance."
    )]
    Unapply {
        #[arg(
            short,
            long,
            value_name = "FILE_PATH",
            help = "The relative or absolute path to the preset configuration file (.yaml) to be unapplied"
        )]
        path: String,
    },
}

fn print_report(stage_results: &[StageResult]) {
    let mut total_time = 0;
    let mut success_count = 0;

    for (_, res) in stage_results.iter().enumerate() {
        total_time += res.time;

        // ANSI escape codes: \x1b[32m = Green, \x1b[31m = Red, \x1b[33m = Yellow, \x1b[0m = Reset
        if res.is_success {
            success_count += 1;
            println!(
                "[\x1b[32mSUCCESS\x1b[0m] Stage #{}: Completed in {} ms",
                res.name, res.time
            );
        } else {
            println!(
                "[\x1b[31mFAILED \x1b[0m] Stage #{}: Failed after {} ms",
                res.name, res.time
            );
            if let Some(ref e) = res.err {
                println!("           \x1b[33mReason:\x1b[0m {:?}", e);
            }
        }
    }

    println!("-------------------------------------");
    let status_color = if success_count == stage_results.len() {
        "\x1b[32m"
    } else {
        "\x1b[31m"
    };
    println!(
        "Summary: {}{}/{}\x1b[0m stages succeeded. Total time: {} ms\n",
        status_color,
        success_count,
        stage_results.len(),
        total_time
    );
}

pub fn execute() {
    let cli = Cli::parse();
    let mut config = ConfigManager::read_config().ok().unwrap_or(Config {
        presets: HashMap::new(),
    });

    #[allow(unreachable_patterns)]
    match cli.command {
        Commands::Apply { path } => {
            let result = apply_preset(&path, &mut config);

            if result.as_ref().is_ok() {
                let stage_results = result.ok().unwrap_or(vec![]);
                print_report(&stage_results);
            } else {
                let err = result.err();
                println!("Error: {:?}", err);
            }
        }

        Commands::Unapply { path } => {
            let result = unapply_preset(&path, &mut config);

            if result.as_ref().is_ok() {
                let stage_results = result.ok().unwrap_or(vec![]);
                print_report(&stage_results);
            } else {
                let err = result.err();
                println!("Error: {:?}", err);
            }
        }

        _ => {}
    }

    let _ = ConfigManager::write_config(&config);
}

mod apply_preset;
mod unapply_preset;
