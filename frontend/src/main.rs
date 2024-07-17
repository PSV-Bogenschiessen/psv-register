use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use chrono::prelude::*;
use common::{bow_type::BowType, class::Class, target_face::TargetFace};
use seed::{prelude::*, *};

#[derive(Serialize, Deserialize)]
struct Model {
    first_name: String,
    last_name: String,
    date_of_birth: NaiveDate,
    mail: InsertedMail,
    bow_type: BowType,
    cls: Option<Class>,
    comment: String,

    possible_target_faces: Vec<TargetFace>,
    selected_target_face: TargetFace,

    submitting: bool,
}

thread_local! {
    static BASE_URL: std::cell::RefCell<Url> = std::cell::RefCell::new(Url::new());
}

impl Model {
    fn new() -> Self {
        let date = NaiveDate::default();
        let cls = Class::classes_for(date)[0];
        Model {
            first_name: String::new(),
            last_name: String::new(),
            date_of_birth: date,
            mail: InsertedMail::Invalid(String::new()),
            bow_type: BowType::Recurve,
            cls: Some(cls),
            comment: String::new(),
            possible_target_faces: TargetFace::for_cls(cls).to_owned(),
            selected_target_face: TargetFace::for_cls(cls)[0],
            submitting: false,
        }
    }
    fn check_and_update_cls(&mut self, orders: &mut impl Orders<Msg>) {
        let available_classes: Vec<Class> = Class::all_classes()
            .filter(|cls| cls.in_range(self.date_of_birth))
            .collect();

        let new_cls = match (self.cls, available_classes.get(0)) {
            (Some(cls), Some(&new)) => {
                if available_classes.contains(&cls) {
                    return;
                } else {
                    Some(new)
                }
            }
            (_, None) => None,
            (None, Some(&new)) => Some(new),
        };

        self.update_target_face();

        orders.send_msg(Msg::ClassChanged(new_cls));
        orders.force_render_now();
    }
    fn update_target_face(&mut self) {
        self.possible_target_faces = if let Some(cls) = self.cls {
            TargetFace::for_cls(cls).to_owned()
        } else {
            Vec::new()
        };
        if !self
            .possible_target_faces
            .contains(&self.selected_target_face)
        {
            self.selected_target_face = *self
                .possible_target_faces
                .get(0)
                .unwrap_or(&TargetFace::M60cm122);
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
                model.check_and_update_cls(orders);
                model.update_target_face();
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

enum Msg {
    FirstNameChanged(String),
    LastNameChanged(String),
    DateOfBirthChanged(String),
    MailChanged(String),
    BowTypeChange(BowType),
    ClassChanged(Option<Class>),
    TargetFaceChanged(TargetFace),
    CommentChanged(String),

    Submit,
    RegistrationFailed(String),
    RegistrationOk,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FirstNameChanged(n) => model.first_name = n,
        Msg::LastNameChanged(n) => model.last_name = n,
        Msg::DateOfBirthChanged(dob) => {
            model.date_of_birth = match chrono::NaiveDate::parse_from_str(&dob, "%Y-%m-%d") {
                Ok(valid) => valid,
                Err(e) => {
                    seed::error!("Date of birth is not valid:", e);
                    Default::default()
                }
            };
            model.check_and_update_cls(orders);
        }
        Msg::MailChanged(mail) => {
            model.mail = if EmailAddress::is_valid(&mail) {
                InsertedMail::Valid(mail)
            } else {
                InsertedMail::Invalid(mail)
            }
        }
        Msg::BowTypeChange(bt) => {
            seed::log!("Selected bow type", bt);
            model.bow_type = bt;
            model.check_and_update_cls(orders);
        }
        Msg::ClassChanged(cls) => {
            seed::log!("Selected cls", cls.map(|cls| cls.name()));
            model.cls = cls;
            model.update_target_face();
        }
        Msg::TargetFaceChanged(tf) => {
            seed::log!("Selected target", tf);
            model.selected_target_face = tf;
        }
        Msg::Submit => {
            model.submitting = true;
            orders.perform_cmd(post_participant(
                common::archer::Archer::new(
                    model.first_name.clone(),
                    model.last_name.clone(),
                    match &model.mail {
                        InsertedMail::Invalid(_) => unreachable!(),
                        InsertedMail::Valid(mail) => EmailAddress::from_str(mail).unwrap(),
                    },
                    model.date_of_birth,
                    model.cls.expect("Submittion only possible if cls is set"),
                    model.selected_target_face,
                    model.comment.clone(),
                )
                .expect("It shouldn't be possible to produce invalid values"),
            ));
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
                mail: model.mail.clone(),
                ..Model::new()
            }
        }
        Msg::CommentChanged(c) => model.comment = c,
    }

    if let Some(session_storage) = window().session_storage().ok().flatten() {
        session_storage
            .set_item("model", &serde_json::to_string(&model).unwrap())
            .unwrap()
    }
}

fn view(model: &Model) -> Node<Msg> {
    let dob = model.date_of_birth;
    ul![
        C!("main"),
        li!("Vorname:"),
        li!(input!(
            attrs!(At::Value => model.first_name),
            input_ev(Ev::Input, Msg::FirstNameChanged)
        )),
        li!("Nachname:"),
        li!(input!(
            attrs!(At::Value => model.last_name),
            input_ev(Ev::Input, Msg::LastNameChanged)
        )),
        li!("Email Adresse:"),
        li!(
            input!(
                attrs!(At::Value => model.mail, At::Type => "email", At::Style => format!("color: {}", if model.mail.is_valid(){"black"} else {"red"}))
            ),
            input_ev(Ev::Input, Msg::MailChanged)
        ),
        li!("Geburtsdatum:"),
        li!(input!(
            attrs!(At::Value => model.date_of_birth, At::Type => "date", ),
            input_ev(Ev::Input, Msg::DateOfBirthChanged)
        )),
        li!(br!()),
        li!("Klasse:"),
        li!(
            attrs!(At::Name => "cls"),
            select!(
                attrs!(At::Name => "Class",At::AutoComplete => "off", At::Required => AtValue::None),
                model.cls.map(|cls| attrs!(At::Value => cls.name())),
                Class::all_classes()
                    .filter(|cls| cls.in_range(model.date_of_birth))
                    .map(|cls| option!(
                        cls.name(),
                        attrs!(At::Value => cls.name()),
                        IF!(Some(cls) == model.cls => attrs!(At::Selected => AtValue::None)),
                        ev(Ev::Input, move |_| { Msg::ClassChanged(Some(cls)) })
                    ))
                    .collect::<Vec<_>>(),
                input_ev(Ev::Input, move |cls_name| {
                    Msg::ClassChanged(Some(
                        Class::classes_for(dob)
                            .into_iter()
                            .find(|cls| cls.name() == cls_name)
                            .unwrap(),
                    ))
                })
            )
        ),
        li!(em!(model.cls.map(|cls| cls.comment()))),
        li!(br!()),
        li!("Scheibe:"),
        li!(model.possible_target_faces.iter().map(|&tf| div![
            input!(
                attrs!(At::Type => "radio", At::Name => "target_face", At::Id => format!("{}", tf)),
                IF!(model.selected_target_face == tf => attrs!(At::Checked => AtValue::None)),
                input_ev(Ev::Input, move |_| Msg::TargetFaceChanged(tf))
            ),
            label!(format!("{}", tf), attrs!(At::For => format!("{}", tf)))
        ]),),
        li!(br!()),
        li!("Kommentar:"),
        li!(textarea!(
            attrs!(At::Value => model.comment),
            input_ev(Ev::Input, Msg::CommentChanged)
        )),
        li!(br!()),
        li!(button!(
            "Anmelden",
            IF!(model.first_name.is_empty() || model.last_name.is_empty() || !model.mail.is_valid() || model.cls.is_none() || model.submitting => attrs!(At::Disabled => AtValue::None)),
            input_ev(Ev::Click, |_| Msg::Submit)
        ))
    ]
}

async fn post_participant(archer: common::archer::Archer) -> Msg {
    let url = BASE_URL.with(|base| base.borrow().clone().set_path(["api", "archers"]));
    let request = Request::new(url.to_string())
        .method(Method::Post)
        .json(&archer)
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
