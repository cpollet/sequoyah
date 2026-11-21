use std::borrow::Cow;

#[allow(unused)]
#[derive(Debug)]
pub struct Graph<'input> {
    pub components: Components<'input>,
    pub overlays: Overlays<'input>,
    pub messages: Messages<'input>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Component<'input> {
    pub kind: ComponentKind,
    pub name: Cow<'input, str>,
    pub identifier: &'input str,
    pub attributes: Vec<Attribute<'input>>,
}

pub type Components<'input> = Vec<Component<'input>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    Component,
    Actor,
    Boundary,
    Control,
    Entity,
    Database,
    Collections,
    Queue,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Message<'input> {
    pub from: &'input str,
    pub arrow: Arrow,
    pub to: &'input str,
    pub attributes: Vec<Attribute<'input>>,
    pub text: Option<Cow<'input, str>>,
    pub activation: Option<Activation<'input>>,
}

pub type Messages<'input> = Vec<Message<'input>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Arrow {
    pub left_dot: bool,
    pub left_head: ArrowHead,
    pub body: ArrowBody,
    pub right_head: ArrowHead,
    pub right_dot: bool,
}

impl Arrow {
    pub fn direction(&self) -> ArrowDirection {
        use ArrowHead::None;
        match (self.left_head, self.right_head) {
            (None, None) => ArrowDirection::None,
            (None, _) => ArrowDirection::Ltr,
            (_, None) => ArrowDirection::Rtl,
            (_, _) => ArrowDirection::Bi,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ArrowHead {
    #[default]
    None,
    Thick,
    Thin,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArrowBody {
    Solid,
    Dash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArrowDirection {
    Ltr,
    Rtl,
    Bi,
    None,
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
pub struct Attribute<'input> {
    pub key: &'input str,
    pub value: Value<'input>,
}

impl<'input> From<Attribute<'input>> for (&'input str, Value<'input>) {
    fn from(value: Attribute<'input>) -> Self {
        (value.key, value.value)
    }
}

impl<'a, 'input> From<&'a Attribute<'input>> for (&'input str, &'a Value<'input>) {
    fn from(value: &'a Attribute<'input>) -> Self {
        (value.key, &value.value)
    }
}

impl<'input> From<(&'input str, Value<'input>)> for Attribute<'input> {
    fn from(value: (&'input str, Value<'input>)) -> Self {
        Attribute {
            key: value.0,
            value: value.1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value<'input> {
    Identifier(&'input str),
    Quoted(Cow<'input, str>),
    HexColor(&'input str),
}

#[allow(unused)]
#[derive(Debug)]
pub struct Activation<'input> {
    pub messages: Vec<Message<'input>>,
    // todo is the message really mandatory? can't we have an arrow with no messages (but a color)?
    pub ret_val: Option<(Vec<Attribute<'input>>, Cow<'input, str>)>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Overlay<'input> {
    pub kind: ComponentKind,
    pub name: Cow<'input, str>,
    pub identifier: &'input str,
    pub components: Vec<&'input str>,
    pub attributes: Vec<Attribute<'input>>,
}

pub type Overlays<'input> = Vec<Overlay<'input>>;
