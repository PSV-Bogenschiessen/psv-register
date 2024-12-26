use std::ops::BitXor;

use chrono::NaiveDate;
use common::{
    bow_type::BowType,
    class::{Class, ClassUpgradeStatus},
    locale::Locale,
    target_face::TargetFace,
};
use rust_i18n::t;
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};

use crate::Msg;

#[derive(Serialize, Deserialize)]
pub enum DoB {
    Vaild(NaiveDate),
    Invalid(String),
}

impl DoB {
    fn is_valid(&self) -> bool {
        match self {
            DoB::Vaild(_) => true,
            DoB::Invalid(_) => false,
        }
    }
}

impl std::fmt::Display for DoB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vaild(dob) => dob.fmt(f),
            Self::Invalid(dob) => dob.fmt(f),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ArcherModel {
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: DoB,
    pub bow_type: BowType,
    pub cls: Option<Class>,
    pub session: u8,

    pub possible_target_faces: Vec<TargetFace>,
    pub selected_target_face: TargetFace,
}

impl ArcherModel {
    pub fn update_target_face(&mut self) {
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
                .first()
                .unwrap_or(&TargetFace::M18Spot);
        }
    }
    pub fn check_and_update_cls(&mut self, index: usize, orders: &mut impl Orders<Msg>) {
        let available_classes = match self.date_of_birth {
            DoB::Vaild(dob) => Class::allowed_classes(self.bow_type, dob)
                .into_iter()
                .map(|(cls, _)| cls)
                .collect::<Vec<_>>(),
            DoB::Invalid(_) => Vec::new(),
        };

        let new_cls = match (self.cls, available_classes.first()) {
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

        orders.send_msg(Msg::ArcherMsg(index, ArcherMsg::ClassChanged(new_cls)));
        orders.force_render_now();
    }

    pub fn ready_for_submission(&self) -> bool {
        !self.first_name.is_empty().bitxor(self.last_name.is_empty())
            && self.cls.is_some()
            && self.date_of_birth.is_valid()
    }
}
impl Default for ArcherModel {
    fn default() -> Self {
        let date = NaiveDate::default();
        let cls = Class::allowed_classes(BowType::Recurve, date)[0].0;
        Self {
            first_name: String::new(),
            last_name: String::new(),
            date_of_birth: DoB::Vaild(date),
            bow_type: BowType::Recurve,
            cls: Some(cls),
            session: 0,
            possible_target_faces: TargetFace::for_cls(cls).to_owned(),
            selected_target_face: TargetFace::for_cls(cls)[0],
        }
    }
}

pub enum ArcherMsg {
    FirstNameChanged(String),
    LastNameChanged(String),
    DateOfBirthChanged(String),
    BowTypeChange(BowType),
    ClassChanged(Option<Class>),
    SessionChanged(u8),
    TargetFaceChanged(TargetFace),
}

pub fn archer_view(model: &ArcherModel, index: usize) -> Node<Msg> {
    let dob = &model.date_of_birth;
    let bow_type = model.bow_type;
    let allowed_classes = match dob {
        DoB::Vaild(dob) => Class::allowed_classes(bow_type, *dob),
        DoB::Invalid(_) => Vec::new(),
    };

    p![
        C!("archer"),
        ul!(
            C!("list flex"),
            li!(
                C!("horizontal"),
                h3!(format!("{} {}:", t!("Archer"), index + 1))
            ),
            li!(
                C!("horizontal"),
                button!(t!("Delete"), input_ev(Ev::Click, move |_| Msg::RemoveArcher(index)))
            )
        ),
        li!(t!("First name")),
        li!(input!(
            attrs!(
                At::Value => model.first_name,
                At::Style =>if model.first_name.is_empty() {"border: 1px solid red"} else {""}
            ),
            input_ev(Ev::Input, move |s| Msg::ArcherMsg(
                index,
                ArcherMsg::FirstNameChanged(s)
            ))
        )),
        li!(t!("Last name")),
        li!(input!(
            attrs!(
                At::Value => model.last_name,
                At::Style =>if model.last_name.is_empty() {"border: 1px solid red"} else {""}
            ),
            input_ev(Ev::Input, move |s| Msg::ArcherMsg(
                index,
                ArcherMsg::LastNameChanged(s)
            ))
        )),
        li!(t!("Date of birth")),
        li!(input!(
            attrs!(
                At::Value => model.date_of_birth,
                At::Type => "date",
                At::Style =>if !model.date_of_birth.is_valid() {"border: 1px solid red"} else {""}
            ),
            input_ev(Ev::Input, move |s| Msg::ArcherMsg(
                index,
                ArcherMsg::DateOfBirthChanged(s)
            ))
        )),
        li!(br!()),
        li!(t!("Session")),
        li!(
            input!(
                attrs!(At::Type => "radio", At::Name => format!("session{}", index), At::Id => format!("session1-{}", index)),
                if model.session == 0 {
                    Some(attrs!("checked" => AtValue::None))
                } else {
                    None
                },
                input_ev(Ev::Input, move |_| Msg::ArcherMsg(
                    index,
                ArcherMsg::SessionChanged(0),
                )),
            ),
            label!(t!("Morning"), attrs!(At::For => format!("session1-{}", index))),
            br!(),

            // input!(
            //     attrs!(At::Type => "radio", At::Name => format!("session{}", index), At::Id => format!("session2-{}", index)),
            //     if model.session == 1 {
            //         Some(attrs!("checked" => AtValue::None))
            //     } else {
            //         None
            //     },
            //     input_ev(Ev::Input, move |_| Msg::ArcherMsg(
            //         index,
            //         ArcherMsg::SessionChanged(1),
            //     )),
            // ),
            // label!(t!("Afternoon"), attrs!(At::For => format!("session2-{}", index))),
            // br!(),

            // input!(
            //     attrs!(At::Type => "radio", At::Name => format!("session{}", index), At::Id => format!("session3-{}", index)),
            //     if model.session == 2 {
            //         Some(attrs!("checked" => AtValue::None))
            //     } else {
            //         None
            //     },
            //     input_ev(Ev::Input, move |_| Msg::ArcherMsg(
            //         index,
            //         ArcherMsg::SessionChanged(2),
            //     )),
            // ),
            // label!(t!("waiting list morning only"), attrs!(At::For => format!("session3-{}", index))),
            // br!(),

            input!(
                attrs!(At::Type => "radio", At::Name => format!("session{}", index), At::Id => format!("session4-{}", index)),
                if model.session == 3 {
                    Some(attrs!("checked" => AtValue::None))
                } else {
                    None
                },
                input_ev(Ev::Input, move |_| Msg::ArcherMsg(
                    index,
                    ArcherMsg::SessionChanged(3),
                )),
            ),
            label!(t!("waiting list afternoon only"), attrs!(At::For => format!("session4-{}", index))),
        ),
        li!(br!()),
        li!(t!("Bow type")),
        li!(
            input!(
                attrs!(At::Type => "radio", At::Name => format!("bow_type{}", index), At::Id => format!("recurve{}",index)),
                if model.bow_type.is_recurve() {
                    Some(attrs!("checked" => AtValue::None))
                } else {
                    None
                },
                input_ev(Ev::Input, move |_| Msg::ArcherMsg(
                    index,
                    ArcherMsg::BowTypeChange(BowType::Recurve)
                ))
            ),
            label!(t!("Recurve"), attrs!(At::For => format!("recurve{}", index))),
            br!(),
            input!(
                attrs!(At::Type => "radio", At::Name => format!("bow_type{}", index), At::Id => format!("blank{}", index)),
                if model.bow_type.is_barebow() {
                    Some(attrs!("checked" => AtValue::None))
                } else {
                    None
                },
                input_ev(Ev::Input, move |_| Msg::ArcherMsg(
                    index,
                    ArcherMsg::BowTypeChange(BowType::Barebow)
                ))
            ),
            label!(t!("Barebow"), attrs!(At::For => format!("blank{}", index))),
            br!(),
            input!(
                attrs!(At::Type => "radio", At::Name => format!("bow_type{}", index), At::Id => format!("compound{}",index), ),
                if model.bow_type.is_compound() {
                    Some(attrs!("checked" => AtValue::None))
                } else {
                    None
                },
                input_ev(Ev::Input, move |_| Msg::ArcherMsg(
                    index,
                    ArcherMsg::BowTypeChange(BowType::Compound)
                ))
            ),
            label!(t!("Compound"), attrs!(At::For => format!("compound{}", index)))
        ),
        li!(br!()),
        li!(t!("Class")),
        li!(
            attrs!(At::Name => "cls"),
            select!(
                attrs!(At::Name => "Class",At::AutoComplete => "off", At::Required => AtValue::None),
                model.cls.map(|cls| attrs!(At::Value => cls.to_string())),
                allowed_classes.clone().into_iter()
                .map(|(cls, upgrade_status)| option!(
                    format!("{}{}",cls.name(Locale::from_str(&rust_i18n::locale()).unwrap()), if upgrade_status == ClassUpgradeStatus::Upgrade  {t!("Upgrade from regular class")} else{"".into()}),
                    attrs!(At::Value => cls.to_string()),
                    IF!(Some(cls) == model.cls => attrs!(At::Selected => AtValue::None)),
                    ev(Ev::Input, move |_| {
                        Msg::ArcherMsg(index, ArcherMsg::ClassChanged(Some(cls)))
                    })
                ))
                .collect::<Vec<_>>(),
                input_ev(Ev::Input, move |cls_id| {
                    Msg::ArcherMsg(
                        index,
                        ArcherMsg::ClassChanged(Some(
                            allowed_classes
                                .into_iter()
                            .map(|(cls, _)| cls)
                                .find(|cls| cls.to_string() == cls_id)
                                .unwrap(),
                        )),
                    )
                })
            )
        ),
        li!(br!()),
        li!(t!("Target")),
        li!(model.possible_target_faces.iter().map(|&tf| div![
            input!(
                attrs!(At::Type => "radio", At::Name => format!("target_face{}", index), At::Id => format!("{}-{}", tf, index)),
                IF!(model.selected_target_face == tf => attrs!(At::Checked => AtValue::None)),
                input_ev(Ev::Input, move |_| Msg::ArcherMsg(
                    index,
                    ArcherMsg::TargetFaceChanged(tf)
                ))
            ),
            label!(format!("{}", tf), attrs!(At::For => format!("{}-{}", tf, index)))
        ]),),
    ]
}

pub fn update_archer(
    msg: ArcherMsg,
    index: usize,
    model: &mut ArcherModel,
    orders: &mut impl Orders<crate::Msg>,
) {
    use ArcherMsg::*;
    match msg {
        FirstNameChanged(n) => model.first_name = n,
        LastNameChanged(n) => model.last_name = n,
        DateOfBirthChanged(dob) => {
            model.date_of_birth = match chrono::NaiveDate::parse_from_str(&dob, "%Y-%m-%d") {
                Ok(valid) => DoB::Vaild(valid),
                Err(e) => {
                    seed::error!("Date of birth is not valid:", e);
                    DoB::Invalid(dob)
                }
            };
            model.check_and_update_cls(index, orders);
        }
        BowTypeChange(bt) => {
            seed::log!("Selected bow type", bt);
            model.bow_type = bt;
            model.check_and_update_cls(index, orders);
        }
        ClassChanged(cls) => {
            seed::log!("Selected cls", cls.map(|cls| cls.to_string()));
            model.cls = cls;
            model.update_target_face();
        }
        TargetFaceChanged(tf) => {
            seed::log!("Selected target", tf);
            model.selected_target_face = tf;
        }
        SessionChanged(session) => {
            seed::log!("Selected session", session);
            model.session = session;
        }
    }
}
