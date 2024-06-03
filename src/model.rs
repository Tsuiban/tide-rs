use crate::{AppWindow, Cli};
use chrono::{Local, NaiveDateTime, NaiveTime, TimeDelta};
use slint::SharedString;
use std::f32::consts::PI;

pub struct Model {
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
    pub start_value: Option<f32>,
    pub end_value: Option<f32>,
    pub interval: Option<f32>,
    pub target_value: Option<f32>,
}

impl Model {
    pub fn is_valid_time(time_string: &String) -> bool {
        for i in time_string.chars() {
            if (i < '0' || i > '9') && i != ':' {
                return false;
            }
        }
        let parts = time_string.split(':').collect::<Vec<&str>>();
        if parts.len() > 3 {
            return false;
        }

        for p in 0..parts.len() {
            let p1 = *parts.get(p).expect("Internal error");
            if p < parts.len() - 1 {
                if p1.len() == 0 || p1.len() > 2 {
                    return false;
                }
            } else {
                if p1.len() > 2 {
                    return false;
                }
            }
        }
        true
    }

    fn naive_time_from_string(time_string: SharedString) -> Option<NaiveTime> {
        let parts = time_string.split(':').collect::<Vec<&str>>();
        let mut time_string = String::new();
        let mut time_partial = String::new();
        for i in parts {
            time_partial.clear();
            if time_string.len() > 0 {
                time_string.push(':');
            }
            while time_partial.len() + i.len() < 2 {
                time_partial.push('0');
            }
            time_partial.push_str(i);
            time_string.push_str(time_partial.as_str());
        }
        let tstr = time_string.as_str();
        match NaiveTime::parse_from_str(tstr, "%H:%M:%S") {
            Ok(t) => Some(t),
            Err(_) => match NaiveTime::parse_from_str(tstr, "%H:%M") {
                Ok(t) => Some(t),
                Err(_) => match NaiveTime::parse_from_str(tstr, "%H") {
                    Ok(t) => Some(t),
                    Err(e) => {
                        eprintln!("Invalid time: {e:?}");
                        None
                    }
                },
            },
        }
    }

    fn float_from_string(float_string: SharedString) -> Option<f32> {
        match float_string.to_string().parse::<f32>() {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }

    pub fn calculate(&self) -> (Option<NaiveDateTime>, String) {
        if !self.start_time.is_some()
            || !self.end_time.is_some()
            || !self.start_value.is_some()
            || !self.end_value.is_some()
        {
            (None, "".to_string())
        } else {
            let target_value_time = if self.target_value.is_some() {
                self.calculate_target_value_time()
            } else {
                None
            };
            let value_table = if self.interval.is_some() {
                self.calculate_value_table()
            } else {
                String::new()
            };

            (target_value_time, value_table)
        }
    }

    fn calculate_target_value_time(&self) -> Option<NaiveDateTime> {
        if self.target_value.is_some() && self.start_value.is_some() && self.end_value.is_some() {
            let tidal_range_fraction = (self.target_value.unwrap() - self.start_value.unwrap())
                / (self.end_value.unwrap() - self.start_value.unwrap());
            let phase_angle_fraction = tidal_range_fraction * 2.0 - 1.0;
            let phase_angle = phase_angle_fraction.asin() + PI / 2.0;
            let time_fraction = phase_angle / PI;
            let time_range = self.end_time.unwrap() - self.start_time.unwrap();
            let elapsed_time = time_range.num_milliseconds() as f32 * time_fraction;
            let time_delta = TimeDelta::milliseconds(elapsed_time as i64);
            let time = self.start_time.unwrap() + time_delta;
            Some(time)
        } else {
            None
        }
    }

    fn calculate_value_table(&self) -> String {
        if self.interval.is_some() && self.end_time.is_some() && self.start_time.is_some() && self.start_value.is_some() && self.end_value.is_some() {
            let mut current_time = TimeDelta::milliseconds(0);
            let interval = TimeDelta::milliseconds((self.interval.unwrap() * 60000.0) as i64);
            let time_range = self.end_time.unwrap() - self.start_time.unwrap();
            let value_range = self.end_value.unwrap() - self.start_value.unwrap();
            let mut results = String::new();
            while current_time <= time_range {
                let entry: String = Model::calculate_table_entry(
                    &current_time,
                    &self.start_time.unwrap(),
                    &time_range,
                    self.start_value.unwrap(),
                    value_range,
                );
                if !results.is_empty() {
                    results.push('\n');
                }
                results.push_str(entry.as_str());
                current_time += interval;
            }
            results
        } else { "".to_string() }
    }

    fn calculate_table_entry(
        current_time: &TimeDelta,
        start_time: &NaiveDateTime,
        time_range: &TimeDelta,
        start_value: f32,
        value_range: f32,
    ) -> String {
        let time_fraction =
            current_time.num_milliseconds() as f32 / time_range.num_milliseconds() as f32;
        let phase_angle = PI * time_fraction - PI / 2.0;
        let amplitude_fraction = phase_angle.sin() / 2.0 + 0.5;
        let amplitude = amplitude_fraction * value_range + start_value;
        let current_time_string = (*start_time + *current_time).format("%H:%M:%S").to_string();
        let entry = format!("{current_time_string} -- {amplitude:.2}");
        entry
    }

    pub fn from_ui(ui: &AppWindow) -> Option<Model> {
        let start_time = Model::naive_time_from_string(ui.get_start_time());
        let local_now = Local::now().date_naive();
        let start_time = NaiveDateTime::new(
            local_now,
            if start_time.is_some() {
                start_time.unwrap()
            } else {
                NaiveTime::from_hms_opt(0, 0, 0).unwrap()
            },
        );
        let end_time = Model::naive_time_from_string(ui.get_end_time());
        let mut end_time = NaiveDateTime::new(
            local_now,
            if end_time.is_some() {
                end_time.unwrap()
            } else {
                NaiveTime::from_hms_opt(0, 0, 0).unwrap()
            },
        );
        if end_time <= start_time {
            end_time += TimeDelta::days(1);
        }
        let start_value = Model::float_from_string(ui.get_start_value());
        let end_value = Model::float_from_string(ui.get_end_value());
        let interval = Model::float_from_string(ui.get_interval());
        let target_value = Model::float_from_string(ui.get_target_value());
        Some(Model {
            start_time: Some(start_time),
            end_time: Some(end_time),
            start_value,
            end_value,
            interval,
            target_value,
        })
    }

    pub fn from_cli(cli: &Cli) -> Option<Model> {
        let now = Local::now().date_naive();
        let start_time = if cli.starttime.is_some() {
            Some(NaiveDateTime::new(now, cli.starttime.unwrap()))
        } else {
            None
        };
        let end_time = if cli.endtime.is_some() {
            let e = NaiveDateTime::new(now, cli.starttime.unwrap());
            if start_time.is_some() && e < start_time.unwrap() {
                Some(e + TimeDelta::days(1))
            } else {
                Some(e)
            }
        } else {
            None
        };
        let start_value = cli.startvalue;
        let end_value = cli.endvalue;
        let interval = cli.interval;
        let target_value = cli.targetvalue;
        Some(Model {
            start_time,
            end_time,
            start_value,
            end_value,
            interval,
            target_value,
        })
    }
    pub fn to_ui(&self, ui: &AppWindow) {
        if self.start_value.is_some() {
            let s = format!("{:.2}", self.start_value.unwrap());
            ui.set_start_value(SharedString::from(s));
        }
        if self.end_value.is_some() {
            let s = format!("{:.2}", self.end_value.unwrap());
            ui.set_end_value(SharedString::from(s));
        }
        if self.interval.is_some() {
            let s = format!("{:.2}", self.interval.unwrap());
            ui.set_interval(SharedString::from(s));
        }
        if self.target_value.is_some() {
            let s = format!("{:.2}", self.target_value.unwrap());
            ui.set_target_value(SharedString::from(s));
        }
        if self.start_time.is_some() {
            let s = self.start_time.unwrap().format("%H:%M:%S").to_string();
            ui.set_start_time(SharedString::from(s));
        }
        if self.end_time.is_some() {
            let s = self.end_time.unwrap().format("%H:%M:%S").to_string();
            ui.set_end_time(SharedString::from(s));
        }
    }
}
