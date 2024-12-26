use crate::{error::*, schema, CONFIG, HANDLEBARS};
use axum::{http::StatusCode, response::IntoResponse, Json};
use common::archer::{Archer, RegisteredArcher};
use common::class::Class;
use common::line_data::CreateArchersPayload;
use diesel::prelude::*;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use log::warn;

#[axum::debug_handler]
pub async fn create_archers(
    Json(payload): Json<CreateArchersPayload>,
) -> Result<impl IntoResponse> {
    // println!("Received {} {}", payload.first_name, payload.last_name);

    if payload.archers.iter().any(|a| [1].contains(&a.session)) {
        return Ok((
            StatusCode::FORBIDDEN,
            Json("Registration closed").into_response(),
        ));
    }

    let total_price: u32 = payload.archers.iter().map(|a| a.class().price()).sum();
    let mail_data = EmailData {
        comment: payload.comment.clone(),
        club: payload.club.clone(),
        mail_address: payload.mail.to_string(),
        name: payload.name.clone(),
        archers: payload
            .archers
            .iter()
            .map(|a| EmailArcher::from(a, payload.locale))
            .collect(),
        total_price: format!("{},{:02}€", total_price / 100, total_price % 100),
    };

    let archers = payload.archers.clone();
    let save_task = tokio::task::spawn_blocking(move || {
        archers
            .into_iter()
            .map(save_archer)
            .collect::<Result<Vec<()>>>()
    });
    let (save, mail) = tokio::join!(save_task, send_registration_mail(mail_data, payload.locale));
    save.unwrap()?;
    mail?;

    Ok((StatusCode::CREATED, Json(payload).into_response()))
}

pub async fn list_archers() -> Result<impl IntoResponse> {
    Ok(Json(get_archers()?))
}

fn save_archer(archer: Archer) -> Result<()> {
    let mut connection = crate::db::establish_connection(&CONFIG.read().database_path);
    connection.transaction(|conn| -> Result<()> {
        let inserted_bib: i32 = diesel::insert_into(schema::archers::table)
            .values(crate::models::InsertableArcher {
                session: archer.session as i32 + 1,
                division: match archer.class() {
                    c if Class::recurve_classes().contains(&c) => "R".to_string(),
                    c if Class::barebow_classes().contains(&c) => "B".to_string(),
                    c if Class::compound_classes().contains(&c) => "C".to_string(),
                    _ => unreachable!(),
                },
                class: format!("{:?}", archer.class()),
                individual_qualification: 1,
                team_qualification: 1,
                individual_final: 1,
                team_final: 1,
                mixed_team_final: 1,
                last_name: archer.last_name.clone(),
                first_name: archer.first_name.clone(),
                gender: None,
                country_code: "TODO".to_string(),
                country_name: archer.club.clone(),
                date_of_birth: archer.date_of_birth().format("%Y-%m-%d").to_string(),
                ..Default::default()
            })
            .returning(schema::archers::bib)
            .get_result(conn)?;

        diesel::insert_into(schema::archer_additions::table)
            .values(crate::models::ArcherAdditions {
                bib: inserted_bib,
                email: archer.mail.as_str().to_owned(),
                target_face: format!("{:?}", archer.target_face()),
                comment: archer.comment,
            })
            .execute(conn)?;

        Ok(())
    })
}

async fn send_registration_mail(
    email_data: EmailData,
    locale: common::locale::Locale,
) -> Result<()> {
    let credentials = Credentials::new(
        CONFIG.read().mail_server.smtp_username.clone(),
        CONFIG.read().mail_server.smtp_password.clone(),
    );
    let email = Message::builder()
        .from(Mailbox::new(
            Some(CONFIG.read().mail_message.sender_name.clone()),
            CONFIG
                .read()
                .mail_message
                .sender_address
                .as_str()
                .parse()
                .map_err(|e| {
                    warn!("received wrong email format: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Falsches Email Format!".to_string(),
                    )
                })
                .unwrap(),
        ))
        .reply_to(Mailbox::new(
            Some("Tobias Edlböck".to_string()),
            "indoor@bogen-psv.de".parse().unwrap(),
        ))
        .to(Mailbox::new(
            Some(email_data.name.clone()),
            email_data.mail_address.parse().unwrap(),
        ))
        .bcc(Mailbox::new(
            Some("Thomas Frank".to_string()),
            "sport@bogen-psv.de".parse().unwrap(),
        ))
        .bcc(Mailbox::new(
            Some("Tobias Edlböck".to_string()),
            "indoor@bogen-psv.de".parse().unwrap(),
        ))
        .header(lettre::message::header::ContentType::TEXT_PLAIN)
        .subject(match locale {
            common::locale::Locale::En => "Registration Confirmation PSV Indoor",
            common::locale::Locale::De => "Anmeldebestätigung PSV Indoor",
        })
        .body(
            HANDLEBARS
                .read()
                .render(
                    match locale {
                        common::locale::Locale::En => "user_mail_en",
                        common::locale::Locale::De => "user_mail",
                    },
                    &email_data,
                )
                .unwrap(),
        )
        .unwrap();

    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&CONFIG.read().mail_server.smtp_server)
            .unwrap()
            .credentials(credentials)
            .build();
    mailer.send(email).await?;
    Ok(())
}

fn get_archers() -> Result<Vec<RegisteredArcher>> {
    use crate::models::*;
    use crate::schema::archers::dsl::*;
    let mut connection = crate::db::establish_connection(&CONFIG.read().database_path);
    let ret = archers.load::<Archer>(&mut connection)?;

    Ok(ret.into_iter().map(|a| a.into()).collect())
}

impl From<crate::models::Archer> for RegisteredArcher {
    fn from(val: crate::models::Archer) -> Self {
        RegisteredArcher {
            first_name: val.first_name,
            last_name: val.last_name,
            class: val.class,
            divison: val.division,
            session: val.session as u8,
            club: val.country_name,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct EmailData {
    comment: String,
    club: String,
    mail_address: String,
    name: String,
    archers: Vec<EmailArcher>,
    total_price: String,
}

#[derive(Debug, serde::Serialize)]
struct EmailArcher {
    first_name: String,
    last_name: String,
    session: String,
    class: String,
    division: String,
    target: String,
    date_of_birth: String,
    price: String,
}

impl EmailArcher {
    fn from(val: &common::archer::Archer, locale: common::locale::Locale) -> Self {
        use Class::*;
        EmailArcher {
            first_name: val.first_name.clone(),
            last_name: val.last_name.clone(),
            session: match val.session {
                0 => match locale {
                    common::locale::Locale::En => "Morning",
                    common::locale::Locale::De => "Vormittag",
                }
                .into(),
                1 => match locale {
                    common::locale::Locale::En => "Afternoon",
                    common::locale::Locale::De => "Nachmittag",
                }
                .into(),
                2 => match locale {
                    common::locale::Locale::En => "waiting list - morning only",
                    common::locale::Locale::De => "Warteliste - nur Vormittags",
                }
                .into(),
                3 => match locale {
                    common::locale::Locale::En => "waiting list - afternoon only",
                    common::locale::Locale::De => "Warteliste - nur Nachmittags",
                }
                .into(),
                _ => format!("{}", val.session),
            },
            class: val.class().name(locale).into(),
            division: match val.class() {
                RUE20M | RUE20W | RU15M | RU15W | RU13M | RU13W | RU11M | RU11W | RU18M | RU18W
                | RU21M | RU21W | RUE49M | RUE49W | RUE65M | RUE65W => "Recurve",

                BUE20M | BUE20W | BU15 | BU21 => match locale {
                    common::locale::Locale::En => "Barebow",
                    common::locale::Locale::De => "Blank",
                },

                CUE20M | CUE20W | CU15 | CU21 | CUE49M | CUE49W | CUE65M | CUE65W => "Compound",
            }
            .into(),
            target: val.target_face().to_string(),
            date_of_birth: val.date_of_birth().format("%Y-%m-%d").to_string(),
            price: format!(
                "{},{:02}€",
                val.class().price() / 100,
                val.class().price() % 100
            ),
        }
    }
}
