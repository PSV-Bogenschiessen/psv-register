mod models;
mod schema;
mod zmi;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use diesel::prelude::*;
use diesel::Connection;
use rand::prelude::*;

#[derive(clap::Parser)]
struct CliArgs {
    /// Path to registration sqlite database
    #[arg(long)]
    db_file: String,

    /// Path to ZMI csv export
    ///
    /// To export a new file open ZMI client:
    /// Select Tab "Listen & Auswertungen".
    /// Press "Auswertungen".
    /// Select "Mitgliederliste".
    /// Click "Export Csv"
    #[arg(long)]
    zmi_file: Option<PathBuf>,

    /// Target face ids
    #[arg(long)]
    target_face_ids: PathBuf,

    /// Asign target
    #[arg(long)]
    assign_targets: bool,
}

#[tokio::main]
async fn main() {
    let CliArgs {
        db_file,
        zmi_file,
        target_face_ids,
        assign_targets,
    } = CliArgs::parse();
    let target_face_to_id = read_target_face_ids(&target_face_ids);

    let mut connection =
        SqliteConnection::establish(&db_file).expect("Couldn't connect to database!");

    let mut archer_with_additions = schema::archers::table
        .inner_join(schema::archer_additions::table)
        .select((
            models::Archer::as_select(),
            models::ArcherAdditions::as_select(),
        ))
        .load::<(models::Archer, models::ArcherAdditions)>(&mut connection)
        .unwrap();

    let zmi_data = if let Some(zmi_file) = zmi_file {
        let zmi_file = std::fs::File::open(zmi_file).unwrap();
        let transcoded = encoding_rs_io::DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::WINDOWS_1252))
            .build(zmi_file);

        let zmi_data: Result<Vec<zmi::ZmiArcher>, _> = csv::ReaderBuilder::new()
            .delimiter(b";"[0])
            .has_headers(true)
            .from_reader(transcoded)
            .deserialize()
            .collect();
        zmi_data.unwrap()
    } else {
        Vec::new()
    };

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b";"[0])
        .has_headers(false)
        .from_writer(std::io::stdout());

    if assign_targets {
        archer_with_additions.shuffle(&mut rand::rngs::StdRng::seed_from_u64(42));
        archer_with_additions
            .sort_by_key(|&(ref ar, ref ad)| (ad.target_face.clone(), ar.class.clone()));

        archer_with_additions
            .iter_mut()
            .fold((3, (None, String::new())), |mut acc, (ar, ad)| {
                if acc.1 != (ad.target_face.clone(), ar.class.clone()) {
                    // new target
                    acc.0 = ((acc.0 / 4) + 1) * 4;
                } else {
                    // just move one up
                    acc.0 += 1;
                }
                acc.1 = (ad.target_face.clone(), ar.class.clone());
                let c = match acc.0 % 4 {
                    0 => 'A',
                    1 => 'B',
                    2 => 'C',
                    3 => 'D',
                    _ => unreachable!(),
                };
                ar.target = format!("{}{}", acc.0 / 4, c);
                acc
            });
    }
    for (arch, _) in &mut archer_with_additions {
        arch.first_name = arch.first_name.trim().to_string();
        arch.last_name = arch.last_name.trim().to_string();

        let hits: Vec<_> = zmi_data
            .iter()
            .filter(|a| a.vorname == arch.first_name && a.namen == arch.last_name)
            .collect();
        if hits.len() == 1 {
            let dob = arch
                .date_of_birth
                .split('-')
                .rev()
                .map(|s| s.parse::<u32>().unwrap().to_string()) // remove leading 0
                .collect::<Vec<_>>()
                .join(".");
            if dob != hits[0].geburtsdatum {
                eprintln!(
                    "DOB for archer {}, {} not matching",
                    arch.last_name, arch.first_name
                );
                eprintln!("zmi {} vs {}", hits[0].geburtsdatum, dob);
            }
            arch.bib = hits[0].passnummer;
        } else if hits.len() >= 1 {
            eprintln!(
                "archer {}, {} appears multiple times in ZMI",
                arch.last_name, arch.first_name
            )
        }
        wtr.serialize(arch).unwrap();
    }
    wtr.flush().unwrap();

    for (arch, additions) in &archer_with_additions {
        println!(
            "##TARGET##;{};{};{};{}",
            arch.bib,
            arch.division,
            arch.class,
            target_face_to_id[additions.target_face.as_ref().unwrap()]
        );
        println!(
            "##EMAIL##{};{}",
            arch.bib,
            additions.email.clone().unwrap_or_default()
        );
    }
}

fn read_target_face_ids(csv: &Path) -> HashMap<String, u8> {
    #[derive(serde::Deserialize)]
    struct DataFormat {
        id: u8,
        target_face: String,
    }

    let data: Result<Vec<DataFormat>, _> = csv::ReaderBuilder::new()
        .delimiter(b";"[0])
        .has_headers(true)
        .from_path(csv)
        .unwrap()
        .deserialize()
        .collect();

    data.unwrap()
        .into_iter()
        .map(
            |DataFormat {
                 id,
                 target_face: name,
             }| (name, id),
        )
        .collect()
}
