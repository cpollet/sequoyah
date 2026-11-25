pub use crate::ast::{Arrow, ComponentKind};
pub use crate::ast::{ArrowBody, ArrowDirection, ArrowHead};
use std::borrow::Cow;

#[allow(unused)]
#[derive(Debug)]
pub struct Graph<'input> {
    pub components: Components<'input>,
    pub overlays: Overlays<'input>,
    pub events: Events<'input>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Component<'input> {
    pub kind: ComponentKind,
    pub name: Cow<'input, str>,
    pub identifier: &'input str,
    pub color: Option<Color<'input>>,
}

pub type Components<'input> = Vec<Component<'input>>;

pub type Color<'input> = &'input str;

#[allow(unused)]
#[derive(Debug)]
pub struct Message<'input> {
    pub from: &'input str,
    pub arrow: Arrow,
    pub to: &'input str,
    pub text: Option<Cow<'input, str>>,
    pub color: Option<Color<'input>>,
    pub activation: Option<Activation<'input>>,
}

#[allow(unused)]
#[derive(Debug)]
pub enum Event<'input> {
    Message(Message<'input>),
    Note(Note<'input>),
    Divider(Divider<'input>),
    Later(Later<'input>),
}

pub type Events<'input> = Vec<Event<'input>>;

#[allow(unused)]
#[derive(Debug)]
pub struct Activation<'input> {
    pub events: Events<'input>,
    pub ret_val: Option<RetVal<'input>>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct RetVal<'input> {
    pub color: Option<Color<'input>>,
    pub text: Cow<'input, str>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Overlay<'input> {
    pub kind: ComponentKind,
    pub name: Cow<'input, str>,
    pub identifier: &'input str,
    pub components: Vec<&'input str>,
    pub color: Option<Color<'input>>,
}

pub type Overlays<'input> = Vec<Overlay<'input>>;

#[allow(unused)]
#[derive(Debug)]
pub struct Note<'input> {
    pub text: Cow<'input, str>,
    pub color: Option<Color<'input>>,
    pub shape: Option<Shape>,
    pub position: Option<Position<'input>>,
}

#[derive(Debug)]
pub enum Shape {
    Default,
    Rectangle,
    Hexagonal,
}

#[derive(Debug)]
pub enum Position<'input> {
    Across,
    Left,
    LeftOf(&'input str),
    Right,
    RightOf(&'input str),
}

#[allow(unused)]
#[derive(Debug)]
pub struct Divider<'input> {
    pub text: Cow<'input, str>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Later<'input> {
    pub text: Cow<'input, str>,
}
