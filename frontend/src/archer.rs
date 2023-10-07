use chrono::NaiveDate;
use common::{bow_type::BowType, class::Class, target_face::TargetFace};
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};

use crate::Msg;

#[derive(Serialize, Deserialize)]
pub struct ArcherModel {
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: NaiveDate,
    pub bow_type: BowType,
    pub cls: Option<Class>,

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
                .get(0)
                .unwrap_or(&TargetFace::M18Spot);
        }
    }
    pub fn check_and_update_cls(&mut self, index: usize, orders: &mut impl Orders<Msg>) {
        let available_classes: Vec<Class> = match self.bow_type {
            BowType::Recurve => Class::recurve_classes(),
            BowType::Compound => Class::compound_classes(),
            BowType::Barebow => Class::barebow_classes(),
        }
        .iter()
        .filter(|cls| cls.in_range(self.date_of_birth))
        .copied()
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

        orders.send_msg(Msg::ArcherMsg(index, ArcherMsg::ClassChanged(new_cls)));
        orders.force_render_now();
    }

    pub fn ready_for_submission(&self) -> bool {
        !self.first_name.is_empty() && !self.last_name.is_empty() && self.cls.is_some()
    }
}
impl Default for ArcherModel {
    fn default() -> Self {
        let date = NaiveDate::default();
        let cls = Class::classes_for(date, BowType::Recurve)[0];
        Self {
            first_name: String::new(),
            last_name: String::new(),
            date_of_birth: date,
            bow_type: BowType::Recurve,
            cls: Some(cls),
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
    TargetFaceChanged(TargetFace),
}

pub fn archer_view(model: &ArcherModel, index: usize) -> Node<Msg> {
    let dob = model.date_of_birth;
    let bow_type = model.bow_type;
    p![
        C!("archer"),
        li!(p![
            h3!(format!("Schütze {}:", index + 1)),
            button!("Löschen", input_ev(Ev::Click, move |_| Msg::RemoveArcher(index)))
        ]),
        li!("Vorname:"),
        li!(input!(
            attrs!(At::Value => model.first_name),
            input_ev(Ev::Input, move |s| Msg::ArcherMsg(
                index,
                ArcherMsg::FirstNameChanged(s)
            ))
        )),
        li!("Nachname:"),
        li!(input!(
            attrs!(At::Value => model.last_name),
            input_ev(Ev::Input, move |s| Msg::ArcherMsg(
                index,
                ArcherMsg::LastNameChanged(s)
            ))
        )),
        li!("Geburtsdatum:"),
        li!(input!(
            attrs!(At::Value => model.date_of_birth, At::Type => format!("date{}", index), ),
            input_ev(Ev::Input, move |s| Msg::ArcherMsg(
                index,
                ArcherMsg::DateOfBirthChanged(s)
            ))
        )),
        li!(br!()),
        li!("Bogenart:"),
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
            label!("Recurve", attrs!(At::For => format!("recurve{}", index))),
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
            label!("Blank", attrs!(At::For => format!("blank{}", index))),
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
            label!("Compound", attrs!(At::For => format!("compound{}", index)))
        ),
        li!(br!()),
        li!("Klasse:"),
        li!(
            attrs!(At::Name => "cls"),
            select!(
                attrs!(At::Name => "Class",At::AutoComplete => "off", At::Required => AtValue::None),
                model.cls.map(|cls| attrs!(At::Value => cls.name())),
                match model.bow_type {
                    BowType::Recurve => Class::recurve_classes(),
                    BowType::Compound => Class::compound_classes(),
                    BowType::Barebow => Class::barebow_classes(),
                }
                .iter()
                .filter(|cls| cls.in_range(model.date_of_birth))
                .map(|cls| option!(
                    cls.name(),
                    attrs!(At::Value => cls.name()),
                    IF!(Some(*cls) == model.cls => attrs!(At::Selected => AtValue::None)),
                    ev(Ev::Input, move |_| {
                        Msg::ArcherMsg(index, ArcherMsg::ClassChanged(Some(*cls)))
                    })
                ))
                .collect::<Vec<_>>(),
                input_ev(Ev::Input, move |cls_name| {
                    Msg::ArcherMsg(
                        index,
                        ArcherMsg::ClassChanged(Some(
                            Class::classes_for(dob, bow_type)
                                .into_iter()
                                .find(|cls| cls.name() == cls_name)
                                .unwrap(),
                        )),
                    )
                })
            )
        ),
        li!(br!()),
        li!("Scheibe:"),
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
                Ok(valid) => valid,
                Err(e) => {
                    seed::error!("Date of birth is not valid:", e);
                    Default::default()
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
            seed::log!("Selected cls", cls.map(|cls| cls.name()));
            model.cls = cls;
            model.update_target_face();
        }
        TargetFaceChanged(tf) => {
            seed::log!("Selected target", tf);
            model.selected_target_face = tf;
        }
    }
}