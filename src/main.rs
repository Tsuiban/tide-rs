mod model;
slint::include_modules!();

use chrono::{NaiveTime};
use clap::Parser;
use model::Model;
use slint::SharedString;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long = "starttime", short = 't')]
    starttime: Option<NaiveTime>,

    #[arg(long = "endtime", short = 'T')]
    endtime: Option<NaiveTime>,

    #[arg(long = "startvalue", short = 'v')]
    startvalue: Option<f32>,

    #[arg(long = "endvalue", short = 'V')]
    endvalue: Option<f32>,

    #[arg(long = "goal", short = 'g')]
    targetvalue: Option<f32>,

    #[arg(long = "interval", short = 'i')]
    interval: Option<f32>,

    #[arg(long = "nogui", short = 'n')]
    nogui: bool,
}

fn main() -> Result<(), slint::PlatformError> {
    let cli = Cli::parse();
    let model = Model::from_cli(&cli).expect("Invalid values on command line.");
    if !cli.nogui {
        let ui = AppWindow::new()?;
        model.to_ui(&ui);

        ui.on_start_time_edited({
            let ui_weak = ui.as_weak();
            move || {
                let ui = ui_weak.unwrap();
                let mut time_string: String = ui.get_start_time().to_string();
                while !time_string.is_empty() && !Model::is_valid_time(&time_string) {
                    _ = time_string.pop();
                }
                ui.set_start_time(SharedString::from(time_string));
            }
        });
        ui.on_end_time_edited({
            let ui_weak = ui.as_weak();
            move || {
                let ui = ui_weak.unwrap();
                let mut time_string: String = ui.get_end_time().to_string();
                while !time_string.is_empty() && !Model::is_valid_time(&time_string) {
                    _ = time_string.pop();
                }
                ui.set_end_time(SharedString::from(time_string));
            }
        });
        ui.on_calculate({
            let ui_weak = ui.as_weak();
            move || {
                let ui = ui_weak.unwrap();
                let model = Model::from_ui(&ui).expect("Internal error fetching values from GUI");
                let (target_time, table_string) = model.calculate();
                if target_time.is_some() {
                    let tstring = target_time.unwrap().format("%H:%M:%S").to_string();
                    ui.set_target_value_time(SharedString::from(tstring));
                } else {
                    ui.set_target_value_time(SharedString::from(""));
                }
                ui.set_value_table(SharedString::from(table_string));
            }
        });
        ui.run()
    } else {
        let model = Model::from_cli(&cli).expect("Insuffient information on command line.");
        let (target_time, table_data) = model.calculate();
        println!("{table_data}");
        if let Some(target_time) = target_time {
            println!("{}", target_time.time().format("%H:%M:%S"));
        }
        Ok(())
    }
}
