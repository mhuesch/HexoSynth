use hexotk::{UIPos, AtomId, wbox};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{
    Container, ContainerData,
    Knob, KnobData,
    Button, ButtonData,
    Text, TextSourceRef, TextData,
    Graph, GraphData,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use crate::matrix::Matrix;

use crate::ui::matrix::MatrixEditorRef;

use crate::dsp::{NodeId, NodeInfo, SAtom, GraphAtomData};

const PANEL_HELP_TEXT_ID      : u32 = 1;
const PANEL_HELP_TEXT_CONT_ID : u32 = 2;
const PANEL_GRAPH_ID          : u32 = 3;

struct GraphAtomDataAdapter<'a> {
    ui: &'a dyn WidgetUI,
}

impl<'a> GraphAtomData for GraphAtomDataAdapter<'a> {
    fn get(&self, node_id: usize, param_idx: u32) -> Option<SAtom> {
        Some(self.ui.atoms().get(AtomId::new(node_id as u32, param_idx))?
             .clone()
             .into())
    }

    fn get_denorm(&self, node_id: usize, param_idx: u32) -> f32 {
        self.ui.atoms()
            .get_denorm(
                AtomId::new(node_id as u32, param_idx))
            .unwrap_or(0.0)
    }
}

pub struct GenericNodeUI {
    dsp_node_id:    NodeId,
    model_node_id:  u32,
    info:           Option<NodeInfo>,
    cont:           Option<Box<WidgetData>>,

    wt_knob_01:    Rc<Knob>,
    wt_knob_11:    Rc<Knob>,
    wt_btn:        Rc<Button>,
    wt_text:       Rc<Text>,
    wt_graph:      Rc<Graph>,

    help_txt:      Rc<TextSourceRef>,
}

impl GenericNodeUI {
    pub fn new() -> Self {
        let wt_knob_01 =
            Rc::new(Knob::new(30.0, 12.0, 9.0));
        let wt_knob_11 =
            Rc::new(Knob::new(30.0, 12.0, 9.0).range_signed());
        let wt_btn   = Rc::new(Button::new(50.0, 12.0));
        let wt_text  = Rc::new(Text::new(10.0));
        let wt_graph = Rc::new(Graph::new(240.0, 100.0));

        Self {
            dsp_node_id:    NodeId::Nop,
            model_node_id:  0,
            info:           None,
            cont:           None,
            help_txt:       Rc::new(TextSourceRef::new(40)),
            wt_knob_01,
            wt_knob_11,
            wt_btn,
            wt_text,
            wt_graph,
        }
    }

    pub fn set_target(&mut self, dsp_node_id: NodeId, model_node_id: u32) {
        self.dsp_node_id   = dsp_node_id;
        self.model_node_id = model_node_id;
        self.info          = Some(NodeInfo::from_node_id(dsp_node_id));

        self.rebuild();
    }

    fn build_atom_input(&self, pos: (u8, u8), idx: usize) -> Option<WidgetData> {
        let param_id = self.dsp_node_id.param_by_idx(idx)?;
        let param_name = param_id.name();

        match param_id.as_atom_def() {
            SAtom::Param(_) => {
                let knob_type =
                    if let Some((min, _max)) = param_id.param_min_max() {
                        if min < 0.0 {
                            self.wt_knob_11.clone()
                        } else {
                            self.wt_knob_01.clone()
                        }
                    } else {
                        // FIXME: Widget type should be determined by the Atom enum!
                        self.wt_knob_01.clone()
                    };

                Some(wbox!(
                    knob_type,
                    AtomId::new(self.model_node_id, idx as u32),
                    center(pos.0, pos.1),
                    KnobData::new(param_name)))
            },
            SAtom::Setting(_) => {
                Some(wbox!(
                    self.wt_btn.clone(),
                    AtomId::new(self.model_node_id, idx as u32),
                    center(pos.0, pos.1),
                    ButtonData::new_toggle(param_name)))
            },
            _ => { None },
        }
    }

    pub fn check_hover_text_for(&mut self, at_id: AtomId) {
        if at_id.node_id() == self.model_node_id {
            if let Some(info) = &self.info {
                if let Some(txt) = info.in_help(at_id.atom_id() as usize) {
                    self.help_txt.set(txt);
                }
            }
        }
    }

    pub fn rebuild(&mut self) {
        let wt_cont = Rc::new(Container::new());

        let mut cd = ContainerData::new();

        println!("REBUILD NODE UI: {} {}",
                 self.dsp_node_id,
                 self.model_node_id);
        cd.contrast_border().new_row();

        for idx in 0..4 {
            if let Some(wd) = self.build_atom_input((4, 6), idx) {
                cd.add(wd);
            }
        }

        let mut txt_cd = ContainerData::new();
        txt_cd
            .level(1)
            .shrink(0.0, 0.0)
//            .contrast_border()
            .border();

        txt_cd.new_row()
            .add(wbox!(self.wt_text,
               AtomId::new(crate::NODE_PANEL_ID, PANEL_HELP_TEXT_ID),
               center(12, 12),
               TextData::new(self.help_txt.clone())));

        if let Some(mut graph_fun) = self.dsp_node_id.graph_fun() {
            let graph_fun =
                Box::new(move |ui: &dyn WidgetUI, init: bool, x: f64| -> f64 {
                    let gd = GraphAtomDataAdapter { ui };
                    graph_fun(&gd, init, x as f32) as f64
                });

            cd.new_row()
              .add(wbox!(self.wt_graph,
                   AtomId::new(crate::NODE_PANEL_ID, PANEL_GRAPH_ID),
                   center(12, 2),
                   GraphData::new(30, graph_fun)));
        }

        cd.new_row()
          .add(wbox!(wt_cont,
               AtomId::new(crate::NODE_PANEL_ID, PANEL_HELP_TEXT_CONT_ID),
               center(12, 2),
               txt_cd));

        // TODO:
        // - detect all the inputs
        // - detect which are atoms (or all atoms)
        // - enumerate all for the AtomId below.
        // - figure out their value ranges for the knobs.
        //   => Maybe define an extra data type for this?
        //      struct InputParamDesc { min, max, type }?
        //      enum UIParamDesc { Knob { min: f32, max: f32 }, ... }
        // - implement a transformation of UIParamDesc to WidgetData.
        // - Implement a Settings input
        // - Implement a sample input with string input.
        // => Then dive off into preset serialization?!
        // => Then dive off into sampler implementation
        //    - With autom. Test!?

        self.cont =
            Some(Box::new(wbox!(
                wt_cont,
                AtomId::new(self.model_node_id, 1),
                center(12, 12), cd)));
    }
}

pub struct NodePanelData {
    #[allow(dead_code)]
    matrix: Arc<Mutex<Matrix>>,

    node_ui: Rc<RefCell<GenericNodeUI>>,

    prev_focus: NodeId,

    editor: MatrixEditorRef,
}

impl NodePanelData {
    pub fn new(_node_id: u32, matrix: Arc<Mutex<Matrix>>, editor: MatrixEditorRef) -> Box<dyn std::any::Any> {
        let node_ui = Rc::new(RefCell::new(GenericNodeUI::new()));
        node_ui.borrow_mut().set_target(NodeId::Sin(0), 1);
        Box::new(Self {
            matrix,
            node_ui,
            editor,
            prev_focus: NodeId::Nop,
        })
    }

    fn check_focus_change(&mut self) {
        let cur_focus = self.editor.get_recent_focus();
        if cur_focus != self.prev_focus {
            self.prev_focus = cur_focus;

            if cur_focus != NodeId::Nop {
                self.node_ui.borrow_mut().set_target(
                    cur_focus,
                    self.matrix.lock().unwrap()
                        .unique_index_for(&cur_focus)
                        .unwrap_or(0)
                        as u32);
            }
        }
//        if prev_focus != cur_focus {
//            self.node_ui.set_target(
//        }
    }
}


#[derive(Debug)]
pub struct NodePanel {
}

impl NodePanel {
    pub fn new() -> Self {
        NodePanel { }
    }
}

impl WidgetType for NodePanel {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos = pos.shrink(UI_PADDING, UI_PADDING);

        data.with(|data: &mut NodePanelData| {
            data.check_focus_change();

            p.rect_fill(UI_BG_CLR, pos.x, pos.y, pos.w, pos.h);

            let pos = pos.shrink(10.0, 10.0);
            p.rect_fill(UI_PRIM_CLR, pos.x, pos.y, pos.w, pos.h);

            let mut node_ui = data.node_ui.borrow_mut();
            if let Some(at_id) = ui.hover_atom_id() {
                node_ui.check_hover_text_for(at_id);
            }

            if let Some(cont) = &mut node_ui.cont {
                cont.draw(ui, p, pos);
            }
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) {
        avail
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        data.with(|data: &mut NodePanelData| {
            let mut node_ui = data.node_ui.borrow_mut();
            if let Some(cont) = &mut node_ui.cont {
                cont.event(ui, ev);
            }
        });
    }
}
