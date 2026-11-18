use crate::ast::{Arrow, ArrowBody, ArrowHead};
use phf::{Map, phf_map};

// let o = ["o", ""];
// let left = ["", "x", "<"];
// let body = ["-", "--"];
// let right = ["", "x", ">"];
//
// let mut i = 0;
// for lo in o {
//     for l    in left {
//         for b in body {
//             for r in right {
//                 for ro in o {
//                     println!("a{i} {lo}{l}{b}{r}{ro} b");
//                     i += 1;
//                 }
//             }
//         }
//     }
// }

pub const ARROWS: Map<&str, Arrow> = phf_map! {
    "o-x" => Arrow {
        left_dot: true, left_head: ArrowHead::None,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "o->o" => Arrow {
        left_dot: true, left_head: ArrowHead::None,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "o->" => Arrow {
        left_dot: true, left_head: ArrowHead::None,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: false,
    },
    "o--x" => Arrow {
        left_dot: true, left_head: ArrowHead::None,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "o-->o" => Arrow {
        left_dot: true, left_head: ArrowHead::None,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "o-->" => Arrow {
        left_dot: true, left_head: ArrowHead::None,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: false,
    },
    "o<-o" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::None, right_dot: true,
    },
    "o<-" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::None, right_dot: false,
    },
    "o<-x" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "o<->o" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "o<->" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: false,
    },
    "o<--o" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::None, right_dot: true,
    },
    "o<--" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::None, right_dot: false,
    },
    "o<--x" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "o<-->o" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "o<-->" => Arrow {
        left_dot: true, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: false,
    },
    "-x" => Arrow {
        left_dot: false, left_head: ArrowHead::None,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "->o" => Arrow {
        left_dot: false, left_head: ArrowHead::None,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "->" => Arrow {
        left_dot: false, left_head: ArrowHead::None,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: false,
    },
    "--x" => Arrow {
        left_dot: false, left_head: ArrowHead::None,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "-->o" => Arrow {
        left_dot: false, left_head: ArrowHead::None,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "-->" => Arrow {
        left_dot: false, left_head: ArrowHead::None,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: false,
    },
    "x-o" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Solid,
        right_head: ArrowHead::None, right_dot: true,
    },
    "x-" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Solid,
        right_head: ArrowHead::None, right_dot: false,
    },
    "x-x" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "x->o" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "x->" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: false,
    },
    "x--o" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Dash,
        right_head: ArrowHead::None, right_dot: true,
    },
    "x--" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Dash,
        right_head: ArrowHead::None, right_dot: false,
    },
    "x--x" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "x-->o" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "x-->" => Arrow {
        left_dot: false, left_head: ArrowHead::Lost,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: false,
    },
    "<-o" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::None, right_dot: true,
    },
    "<-" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::None, right_dot: false,
    },
    "<-x" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "<->o" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "<->" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Solid,
        right_head: ArrowHead::Thick, right_dot: false,
    },
    "<--o" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::None, right_dot: true,
    },
    "<--" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::None, right_dot: false,
    },
    "<--x" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Lost, right_dot: false,
    },
    "<-->o" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: true,
    },
    "<-->" => Arrow {
        left_dot: false, left_head: ArrowHead::Thick,
        body: ArrowBody::Dash,
        right_head: ArrowHead::Thick, right_dot: false,
    },
};
