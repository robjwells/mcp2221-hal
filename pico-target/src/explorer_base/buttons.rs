#![allow(unused)]

use embassy_futures::select::SelectArray;
use embassy_rp::{
    gpio::{Input, Pull},
    peripherals::{PIN_12, PIN_13, PIN_14, PIN_15},
};

// Display and graphics
#[derive(Clone, Copy, defmt::Format)]
pub struct ButtonsPressed {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
}

impl ButtonsPressed {
    pub fn any_pressed(&self) -> bool {
        self.a || self.b || self.x || self.y
    }
}

pub struct Buttons {
    a: Input<'static>,
    b: Input<'static>,
    x: Input<'static>,
    y: Input<'static>,
}

impl Buttons {
    pub fn new(a: PIN_12, b: PIN_13, x: PIN_14, y: PIN_15) -> Self {
        let mut a = Input::new(a, Pull::Up);
        let mut b = Input::new(b, Pull::Up);
        let mut x = Input::new(x, Pull::Up);
        let mut y = Input::new(y, Pull::Up);
        a.set_schmitt(true);
        b.set_schmitt(true);
        x.set_schmitt(true);
        y.set_schmitt(true);
        Self { a, b, x, y }
    }

    pub fn wait_for_any_edge(&mut self) -> SelectArray<impl Future<Output = ()>, 4> {
        embassy_futures::select::select_array([
            self.a.wait_for_any_edge(),
            self.b.wait_for_any_edge(),
            self.x.wait_for_any_edge(),
            self.y.wait_for_any_edge(),
        ])
    }

    pub fn state(&self) -> ButtonsPressed {
        ButtonsPressed {
            a: self.a.is_low(),
            b: self.b.is_low(),
            x: self.x.is_low(),
            y: self.y.is_low(),
        }
    }
}
