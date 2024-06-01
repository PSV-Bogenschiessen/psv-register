use crate::{error::*, schema, CONFIG, HANDLEBARS};
use axum::{http::StatusCode, response::IntoResponse, Json};
use common::archer::{Archer, RegisteredArcher};
use common::class::Class;
use diesel::prelude::*;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use log::warn;
use std::collections::BTreeMap;

#[axum::debug_handler]
pub async fn create_archer(Json(payload): Json<Archer>) -> Result<impl IntoResponse> {
    println!("Received {} {}", payload.first_name, payload.last_name);

    let email_data = BTreeMap::from([
        ("first_name", payload.first_name.clone()),
        ("last_name", payload.last_name.clone()),
        (
            "date_of_birth",
            payload.date_of_birth().format("%d.%m.%Y").to_string(),
        ),
        ("class", payload.class().name().to_owned()),
        ("target_face", payload.target_face().to_string()),
        ("comment", payload.comment.clone()),
    ]);

    let archer = payload.clone();
    let save_task = tokio::task::spawn_blocking(move || save_archer(archer));
    let (save, mail) = tokio::join!(save_task, send_registration_mail(&payload, email_data));

    save.unwrap()?;
    mail?;

    Ok((StatusCode::CREATED, Json(payload)))
}

pub async fn list_archers() -> Result<impl IntoResponse> {
    Ok(Json(get_archers()?))
}

fn save_archer(archer: Archer) -> Result<()> {
    let mut connection = crate::db::establish_connection();
    connection.transaction(|conn| -> Result<()> {
        let inserted_bib: i64 = diesel::insert_into(schema::archers::table)
            .values(crate::models::InsertableArcher {
                session: 1,
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
                country_code: "PSV".to_string(),
                country_name: "PSV München".to_string(),
                ..Default::default()
            })
            .returning(schema::archers::bib)
            .get_result(conn)?;

        diesel::insert_into(schema::archer_additions::table)
            .values(crate::models::ArcherAdditions {
                bib: inserted_bib,
                email: Some(archer.mail.as_str().to_owned()),
                target_face: Some(format!("{:?}", archer.target_face())),
                comment: Some(archer.comment),
            })
            .execute(conn)?;

        Ok(())
    })
}

async fn send_registration_mail(
    archer: &Archer,
    email_data: BTreeMap<&'static str, String>,
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
        .to(Mailbox::new(
            Some(format!("{} {}", archer.first_name, archer.last_name)),
            archer.mail.as_str().parse().unwrap(),
        ))
        .bcc(Mailbox::new(
            Some("Thomas Frank".to_string()),
            "sport@bogen-psv.de".parse().unwrap(),
        ))
        .header(lettre::message::header::ContentType::TEXT_PLAIN)
        .subject(&CONFIG.read().mail_message.subject)
        .body(HANDLEBARS.read().render("user_mail", &email_data).unwrap())
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
    let mut connection = crate::db::establish_connection();
    let ret = archers.load::<Archer>(&mut connection)?;

    Ok(ret.into_iter().map(|a| a.into()).collect())
}

impl From<crate::models::Archer> for RegisteredArcher {
    fn from(val: crate::models::Archer) -> Self {
        RegisteredArcher {
            first_name: val.first_name,
            last_name: val.last_name,
            class: val.class.parse().unwrap(),
        }
    }
}
