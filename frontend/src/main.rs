mod archer;
mod registrator;

use archer::ArcherModel;
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use seed::{prelude::*, *};

#[derive(Serialize, Deserialize)]
struct Model {
    registrator: Registrator,
    archers: Vec<ArcherModel>,

    submitting: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Registrator {
    name: String,
    mail: InsertedMail,
    comment: String,
}

thread_local! {
    static BASE_URL: std::cell::RefCell<Url> = std::cell::RefCell::new(Url::new());
}

impl Model {
    fn new() -> Self {
        Model {
            registrator: Registrator {
                name: String::new(),
                mail: InsertedMail::Invalid(String::new()),
                comment: String::new(),
            },
            archers: vec![ArcherModel::default()],
            submitting: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum InsertedMail {
    Invalid(String),
    Valid(String),
}

impl InsertedMail {
    fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }
}

impl Default for InsertedMail {
    fn default() -> Self {
        Self::Invalid(String::new())
    }
}
impl Display for InsertedMail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InsertedMail::Invalid(s) => s.fmt(f),
            InsertedMail::Valid(s) => s.fmt(f),
        }
    }
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    BASE_URL.with(|base_url| {
        *base_url.borrow_mut() = url.to_base_url();
    });
    let window = window();
    let Some(session_storage) = window.session_storage().ok().flatten() else {
        seed::log!("Couldn't load session storage");
        return Model::new();
    };
    if let Some(ser_model) = session_storage.get_item("model").unwrap() {
        match serde_json::from_str::<Model>(&ser_model) {
            Ok(mut model) => {
                model.submitting = false;
                for (index, archer) in model.archers.iter_mut().enumerate() {
                    archer.check_and_update_cls(index, orders);
                    archer.update_target_face();
                }
                model
            }
            Err(_) => {
                seed::error!("Failed to load stored session");
                Model::new()
            }
        }
    } else {
        Model::new()
    }
}

pub enum Msg {
    ArcherMsg(usize, archer::ArcherMsg),
    AddArcher,
    RemoveArcher(usize),

    NameChanged(String),
    MailChanged(String),
    CommentChanged(String),

    Submit,
    RegistrationFailed(String),
    RegistrationOk,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::MailChanged(mail) => {
            model.registrator.mail = if EmailAddress::is_valid(&mail) {
                InsertedMail::Valid(mail)
            } else {
                InsertedMail::Invalid(mail)
            }
        }
        Msg::Submit => {
            model.submitting = true;
            let mail = match &model.registrator.mail {
                InsertedMail::Invalid(_) => unreachable!(),
                InsertedMail::Valid(mail) => EmailAddress::from_str(mail).unwrap(),
            };
            orders.perform_cmd(post_participants(common::line_data::CreateArchersPayload {
                name: model.registrator.name.clone(),
                mail: mail.clone(),
                comment: model.registrator.comment.clone(),
                archers: model
                    .archers
                    .iter()
                    .filter(|a| !a.first_name.is_empty() && !a.last_name.is_empty())
                    .map(|a| {
                        common::archer::Archer::new(
                            a.first_name.clone(),
                            a.last_name.clone(),
                            mail.clone(),
                            match a.date_of_birth {
                                archer::DoB::Vaild(dob) => dob,
                                archer::DoB::Invalid(_) => {
                                    unreachable!("Submission only impossible if dob is valid")
                                }
                            },
                            a.cls.expect("Submission only possible if class is set"),
                            a.selected_target_face,
                            model.registrator.comment.clone(),
                        )
                        .expect("It shouldn't be possible to produce invalid values")
                    })
                    .collect(),
            }));
        }
        Msg::RegistrationFailed(err) => {
            seed::window()
                .alert_with_message(&format!("Fehler! {err:?}"))
                .ok();
            seed::error!("Submission failed!", err);
            model.submitting = false;
        }
        Msg::RegistrationOk => {
            seed::window()
                .alert_with_message("Anmeldung erfolgreich. Bestätigungsmail wurde abgeschickt.")
                .ok();
            seed::log!("Submission ok!");
            *model = Model {
                registrator: model.registrator.clone(),
                ..Model::new()
            }
        }
        Msg::CommentChanged(c) => model.registrator.comment = c,
        Msg::ArcherMsg(index, a_msg) => {
            archer::update_archer(a_msg, index, &mut model.archers[index], orders)
        }
        Msg::AddArcher => {
            model.archers.push(ArcherModel::default());
        }
        Msg::RemoveArcher(index) => {
            model.archers.remove(index);
        }
        Msg::NameChanged(name) => {
            model.registrator.name = name;
        }
    }

    if let Some(session_storage) = window().session_storage().ok().flatten() {
        session_storage
            .set_item("model", &serde_json::to_string(&model).unwrap())
            .unwrap()
    }
}

fn view(model: &Model) -> Node<Msg> {
    // let dob = model.date_of_birth;
    // let bow_type = model.bow_type;
    ul![
        C!("list"),
        registrator::view_registrator(&model.registrator),
        hr!(),
        model
            .archers
            .iter()
            .enumerate()
            .map(|(index, archer)| { p!(li!(archer::archer_view(archer, index)), hr!()) }),
        li!(button!(
            "Schützen Hinzufügen",
            input_ev(Ev::Click, |_| Msg::AddArcher)
        )),
        li!(br!()),
        li!(button!(
            "Anmeldung Einreichen",
            IF!(model.archers.is_empty() || model.archers.iter().any(|a| !a.ready_for_submission()) || !model.registrator.mail.is_valid() || model.submitting => attrs!(At::Disabled => AtValue::None)),
            input_ev(Ev::Click, |_| Msg::Submit)
        ))
    ]
}

async fn post_participants(data: common::line_data::CreateArchersPayload) -> Msg {
    let url = BASE_URL.with(|base| base.borrow().clone().set_path(["api", "archers"]));
    let request = Request::new(url.to_string())
        .method(Method::Post)
        .json(&data)
        .unwrap();
    let response = match fetch(request).await {
        Ok(r) => r,
        Err(e) => return Msg::RegistrationFailed(format!("{e:?}")),
    };
    let text = response.text().await;
    match response.check_status() {
        Ok(_) => Msg::RegistrationOk,
        Err(e) => {
            seed::log!(e);
            Msg::RegistrationFailed(text.unwrap_or(format!("{e:?}")))
        }
    }
}

pub fn main() {
    App::start("app", init, update, view);
}
