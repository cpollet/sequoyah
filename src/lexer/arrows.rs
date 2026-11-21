use crate::ast::{Arrow, ArrowBody, ArrowHead};
use phf::{Map, phf_map};

#[allow(dead_code)]
pub fn gen_arrows() {
    let o = [("o", true), ("", false)];
    let left = [
        ("", ArrowHead::None),
        ("x", ArrowHead::Lost),
        ("<", ArrowHead::Thick),
    ];
    let body = [("-", ArrowBody::Solid), ("--", ArrowBody::Dash)];
    let right = [
        ("", ArrowHead::None),
        ("x", ArrowHead::Lost),
        (">", ArrowHead::Thick),
    ];

    let mut i = 0;
    for lo in o {
        for l in left {
            if lo.1 && matches!(l.1, ArrowHead::Lost) {
                continue;
            }
            for b in body {
                for r in right {
                    for ro in o {
                        if matches!(l.1, ArrowHead::None) && matches!(r.1, ArrowHead::None) {
                            continue;
                        }
                        if ro.1 && matches!(r.1, ArrowHead::Lost) {
                            continue;
                        }
                        println!(
                            "a{i} {lo}{l}{b}{r}{ro} b",
                            lo = lo.0,
                            l = l.0,
                            b = b.0,
                            r = r.0,
                            ro = ro.0
                        );
                        i += 1;
                    }
                }
            }
        }
    }
}

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
