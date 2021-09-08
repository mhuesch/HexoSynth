// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use tuix::*;

mod hexo_consts;
mod painter;
mod hexgrid;
mod rect;

use painter::FemtovgPainter;
use hexgrid::{HexGrid, HexGridModel, HexCell, HexDir, HexEdge};
use hexo_consts::MButton;

use std::rc::Rc;
use std::cell::RefCell;

struct TestGridModel {
}

impl TestGridModel {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl HexGridModel for TestGridModel {
    fn width(&self) -> usize { 16 }
    fn height(&self) -> usize { 16 }
    fn cell_visible(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }
    fn cell_empty(&self, x: usize, y: usize) -> bool {
        !(x < self.width() && y < self.height())
    }
    fn cell_color(&self, x: usize, y: usize) -> u8 { 0 }
    fn cell_label<'a>(&self, x: usize, y: usize, out: &'a mut [u8])
        -> Option<HexCell<'a>>
    {
        None
    }

    /// Edge: 0 top-right, 1 bottom-right, 2 bottom, 3 bottom-left, 4 top-left, 5 top
    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, out: &'a mut [u8])
        -> Option<(&'a str, HexEdge)>
    {
        None
    }

    fn cell_click(&self, x: usize, y: usize, btn: MButton, modkey: bool) {
    }

    fn cell_hover(&self, x: usize, y: usize) {
    }
}

#[derive(Lens)]
pub struct UIState {
    grid_1: Rc<RefCell<dyn HexGridModel>>,
    grid_2: Rc<RefCell<dyn HexGridModel>>,
}

impl Model for UIState {
}

fn main() {
    let mut app =
        Application::new(
            WindowDescription::new(),
            |state, window| {
                let ui_state =
                    UIState {
                        grid_1: Rc::new(RefCell::new(TestGridModel::new())),
                        grid_2: Rc::new(RefCell::new(TestGridModel::new())),
                    };

                let app_data = ui_state.build(state, window);

                let row = Row::new().build(state, app_data, |builder| builder);

                let hex =
                    HexGrid::new(1, 64.0)
                        .bind(UIState::grid_1, |value| value.clone())
                        .build(state, row, |builder| builder);
                let hex2 =
                    HexGrid::new(2, 72.0)
                        .bind(UIState::grid_2, |value| value.clone())
                        .build(state, row, |builder| builder);
            });
    app.run();
}
