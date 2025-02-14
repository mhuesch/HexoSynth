// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//use crate::arg_chk;
use wlambda::*;
use hexodsp::{Matrix, NodeId, SAtom, dsp::GraphFun, dsp::GraphAtomData};
use hexotk::GraphModel;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

struct NodeGraphAtomData {
    matrix:     Arc<Mutex<Matrix>>,
    node_id:    NodeId,
}

impl GraphAtomData for NodeGraphAtomData {
    fn get(&self, param_idx: u32) -> Option<SAtom> {
        let m = self.matrix.lock().expect("Matrix lockable");
        let pid = self.node_id.param_by_idx(param_idx as usize)?;
        m.get_param(&pid)
    }
    fn get_denorm(&self, param_idx: u32) -> f32 {
        let m = self.matrix.lock().expect("Matrix lockable");
        if let Some(pid) = self.node_id.param_by_idx(param_idx as usize) {
            if let Some(at) = m.get_param(&pid) {
                pid.denorm(at.f())
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    fn get_norm(&self, param_idx: u32) -> f32 {
        let m = self.matrix.lock().expect("Matrix lockable");
        if let Some(pid) = self.node_id.param_by_idx(param_idx as usize) {
            if let Some(at) = m.get_param(&pid) {
                at.f()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    fn get_phase(&self) -> f32 {
        let m = self.matrix.lock().expect("Matrix lockable");
        m.phase_value_for(&self.node_id)
    }
    fn get_led(&self) -> f32 {
        let m = self.matrix.lock().expect("Matrix lockable");
        m.led_value_for(&self.node_id)
    }
}

struct NodeGraphModel {
    matrix:   Arc<Mutex<Matrix>>,
    nga_data: Box<dyn GraphAtomData>,
    fun:      Option<GraphFun>,
}

impl GraphModel for NodeGraphModel {
    fn get_generation(&self) -> u64 {
        let m = self.matrix.lock().expect("Matrix lockable");
        m.get_generation() as u64
    }
    fn f(&mut self, init: bool, x: f64, x_next: f64) -> f64 {
        if let Some(fun) = &mut self.fun {
            (*fun)(&*self.nga_data, init, x as f32, x_next as f32) as f64
        } else {
            0.0
        }
    }
    fn vline1_pos(&self) -> Option<f64> { None }
    fn vline2_pos(&self) -> Option<f64> { None }
}

#[derive(Clone)]
pub struct VGraphModel(Rc<RefCell<dyn GraphModel>>);

impl VGraphModel {
    pub fn new(matrix: Arc<Mutex<Matrix>>, node_id: NodeId) -> Self {
        Self(Rc::new(RefCell::new(NodeGraphModel {
            nga_data: Box::new(
                NodeGraphAtomData {
                    matrix: matrix.clone(),
                    node_id: node_id.clone(),
                },
            ),
            fun: node_id.graph_fun(),
            matrix,
        })))
    }
}

impl VValUserData for VGraphModel {
    fn s(&self) -> String { format!("$<UI::GraphModel>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, _key: &str, _env: &mut Env)
        -> Result<VVal, StackAction>
    {
        Ok(VVal::None)
    }
}

pub fn vv2graph_model(mut v: VVal) -> Option<Rc<RefCell<dyn GraphModel>>> {
    v.with_usr_ref(|model: &mut VGraphModel| model.0.clone())
}
