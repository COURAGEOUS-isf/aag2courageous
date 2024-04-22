use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::{anyhow, Context};
use clap::CommandFactory;
use courageous_format::{Alarm, Position3d, Track, TrackingRecord, Version};
use nmea::{
    sentences::{GgaData, RmcData},
    NmeaSentence,
};

mod clap_util;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Input {
    /// Path to the file to convert.
    input_path: PathBuf,

    /// The location of the C-UAS surveilling the UAS whose position is being logged.
    #[arg(value_parser = clap_util::Position3dParser)]
    static_cuas_location: Position3d,

    /// Path of the resulting file. [default: {input_path}.json]
    #[arg(short)]
    output_path: Option<PathBuf>,

    /// Pretty-print the resulting JSON.
    #[arg(long, default_value_t = false)]
    prettyprint: bool,

    /// The system name specified in the resulting COURAGEOUS file.
    #[arg(long, default_value_t = {"Unknown".to_owned()})]
    system_name: String,

    /// The vendor name specified in the resulting COURAGEOUS file.
    #[arg(long, default_value_t = {"Unknown".to_owned()})]
    vendor_name: String,
}

fn main() -> anyhow::Result<()> {
    let input = Input::command()
        .help_template(include_str!("help_template"))
        .get_matches();

    let input_path = input.get_one::<PathBuf>("input_path").unwrap();
    let output_path = input
        .get_one::<PathBuf>("output_path")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| input_path.with_extension("json"));
    let static_cuas_location = *input.get_one::<Position3d>("static_cuas_location").unwrap();
    let prettyprint_output = input.get_flag("prettyprint");
    let system_name = input.get_one::<String>("system_name").unwrap().clone();
    let vendor_name = input.get_one::<String>("vendor_name").unwrap().clone();

    let input_file = BufReader::new(
        File::open(input_path)
            .with_context(|| format!("Failed to read input file at {}", input_path.display()))?,
    );
    let output_file =
        BufWriter::new(File::create(&output_path).with_context(|| {
            format!("Failed to write output file at {}", output_path.display())
        })?);

    // Aaronia GPRMC / GPGGA messages may be desynchronized by a second sometimes: Resynchronize them
    let lines = input_file.lines().collect::<Result<Vec<String>, _>>()?;
    let mut paired_records: HashMap<chrono::NaiveTime, (Option<RmcData>, Option<GgaData>)> =
        HashMap::new();
    for line in lines {
        if line.starts_with("$PAAG") {
            continue;
        }

        // TODO: Fork nmea and make Error statically lived
        let nmea_sentence =
            nmea::parse_nmea_sentence(&line).map_err(|err| anyhow!(err.to_string()))?;
        // TODO: Fork nmea and add Clone & Copy to NmeaSentence
        let nmea_sentence_2 = NmeaSentence {
            checksum: nmea_sentence.checksum,
            data: nmea_sentence.data,
            message_id: nmea_sentence.message_id,
            talker_id: nmea_sentence.talker_id,
        };
        if let Ok(rmc) = nmea::sentences::parse_rmc(nmea_sentence) {
            if let Some(time) = rmc.fix_time {
                paired_records.entry(time).or_default().0 = Some(rmc);
            }
        } else if let Ok(gga) = nmea::sentences::parse_gga(nmea_sentence_2) {
            if let Some(time) = gga.fix_time {
                paired_records.entry(time).or_default().1 = Some(gga);
            }
        }
    }

    let records = paired_records
        .into_iter()
        .filter_map(|(_, (rmc, gga))| match (rmc, gga) {
            (Some(rmc), Some(gga)) => Some((rmc, gga)),
            _ => None,
        })
        .enumerate()
        .filter_map(|(record_idx, (rmc, gga))| -> Option<TrackingRecord> {
            let Some(date) = rmc.fix_date else {
                return None;
            };
            let (Some(time), Some(lat), Some(lon), Some(height)) =
                (gga.fix_time, gga.latitude, gga.longitude, gga.altitude)
            else {
                return None;
            };
            let time = date
                .and_time(time)
                .and_utc()
                .signed_duration_since(chrono::DateTime::UNIX_EPOCH)
                .num_seconds() as u64;
            // let (dir_sin, dir_cos) = (direction as f64).to_radians().sin_cos();
            // let (speed_x, speed_y) = (speed * dir_cos, speed * dir_sin);

            let pos = Position3d {
                lat,
                lon,
                height: height as f64,
            };

            Some(TrackingRecord {
                alarm: Alarm {
                    active: false,
                    certainty: 0.,
                },
                classification: courageous_format::Classification::Uav,
                location: courageous_format::Location::Position3d(pos),
                record_number: record_idx as u64,
                time,
                // TODO: Velocity
                velocity: None, // We have speed over ground and true course, but no speed on the up axis? Or does speed over ground include up speed?
                identification: None,
                cuas_location: None,
            })
        })
        .collect::<Vec<_>>();

    let document = courageous_format::Document {
        detection: vec![],
        static_cuas_location,
        tracks: vec![Track {
            name: Some(format!(
                "Aaronia GPS track '{}'",
                input_path
                    .file_name()
                    .map(|str| str.to_string_lossy())
                    .unwrap_or(Cow::Borrowed("no filename"))
            )),
            uas_id: 1,
            records,
            uav_home_location: None,
        }],
        system_name,
        vendor_name,
        version: Version::current(),
    };

    if prettyprint_output {
        serde_json::to_writer_pretty(output_file, &document)?;
    } else {
        serde_json::to_writer(output_file, &document)?;
    }

    Ok(())
}
