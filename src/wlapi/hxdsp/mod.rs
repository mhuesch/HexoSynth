// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

pub mod node_info;
pub mod param;
pub mod atom;
pub mod matrix;
pub mod matrix_recorder;
pub mod grid_model;
pub mod octave_keys;
pub mod graph;
pub mod graph_minmax;

pub use node_info::*;
pub use param::*;
pub use atom::*;
pub use matrix::*;
pub use matrix_recorder::*;
pub use grid_model::*;
pub use octave_keys::*;
pub use graph::*;
pub use graph_minmax::*;

use wlambda::*;
use hexodsp::{NodeId};
use hexodsp::dsp::{UICategory};

pub fn vv2node_id(v: &VVal) -> NodeId {
    let node_id = v.v_(0).with_s_ref(|s| NodeId::from_str(s));
    node_id.to_instance(v.v_i(1) as usize)
}

pub fn node_id2vv(nid: NodeId) -> VVal {
    VVal::pair(VVal::new_str(nid.name()), VVal::Int(nid.instance() as i64))
}

fn ui_category2str(cat: UICategory) -> &'static str {
    match cat {
        UICategory::None   => "none",
        UICategory::Osc    => "Osc",
        UICategory::Mod    => "Mod",
        UICategory::NtoM   => "NtoM",
        UICategory::Signal => "Signal",
        UICategory::Ctrl   => "Ctrl",
        UICategory::IOUtil => "IOUtil",
    }
}

pub fn setup_node_id_module() -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    st.fun(
        "list_all", move |_env: &mut Env, _argc: usize| {
            let ids = VVal::vec();

            for nid in hexodsp::dsp::ALL_NODE_IDS.iter() {
                ids.push(VVal::new_str(nid.name()));
            }

            Ok(ids)
        }, Some(0), Some(0), false);

    st.fun(
        "ui_category_list", move |_env: &mut Env, _argc: usize| {
            let cats = VVal::vec();

            for cat in [
                UICategory::None,
                UICategory::Osc,
                UICategory::Mod,
                UICategory::NtoM,
                UICategory::Signal,
                UICategory::Ctrl,
                UICategory::IOUtil
            ]
            {
                cats.push(VVal::pair(
                    VVal::new_sym(ui_category2str(cat)),
                    VVal::Int(cat.default_color_idx() as i64)));
            }

            Ok(cats)
        }, Some(0), Some(0), false);

    st.fun(
        "ui_category_node_id_map", move |_env: &mut Env, _argc: usize| {
            let m = VVal::map();

            for cat in [
                UICategory::Osc,
                UICategory::Mod,
                UICategory::NtoM,
                UICategory::Signal,
                UICategory::Ctrl,
                UICategory::IOUtil
            ]
            {
                let v = VVal::vec();
                cat.get_node_ids(0, |nid| { v.push(node_id2vv(nid)); });
                let _ = m.set_key_str(ui_category2str(cat), v);
            }

            Ok(m)
        }, Some(0), Some(0), false);

    st.fun(
        "ui_category", move |env: &mut Env, _argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            Ok(VVal::new_sym(ui_category2str(nid.ui_category())))
        }, Some(1), Some(1), false);

    st.fun(
        "instance", move |env: &mut Env, _argc: usize| {
            Ok(VVal::Int(vv2node_id(&env.arg(0)).instance() as i64))
        }, Some(1), Some(1), false);

    st.fun(
        "name", move |env: &mut Env, _argc: usize| {
            Ok(VVal::new_str(vv2node_id(&env.arg(0)).name()))
        }, Some(1), Some(1), false);

    st.fun(
        "label", move |env: &mut Env, _argc: usize| {
            Ok(VVal::new_str(vv2node_id(&env.arg(0)).label()))
        }, Some(1), Some(1), false);

    let mut info_map : std::collections::HashMap<String, VVal> =
        std::collections::HashMap::new();

    for nid in hexodsp::dsp::ALL_NODE_IDS.iter() {
        info_map.insert(
            nid.name().to_string(),
            VVal::new_usr(VValNodeInfo::new(*nid)));
    }

    st.fun(
        "info", move |env: &mut Env, _argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            Ok(info_map.get(nid.name()).map_or(VVal::None, |v| v.clone()))
        }, Some(1), Some(1), false);

    st.fun(
        "eq_variant", move |env: &mut Env, _argc: usize| {
            Ok(VVal::Bol(
                            vv2node_id(&env.arg(0))
                .eq_variant(&vv2node_id(&env.arg(1)))))
        }, Some(2), Some(2), false);

    st.fun(
        "param_by_idx", move |env: &mut Env, _argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            let param = nid.param_by_idx(env.arg(1).i() as usize);

            Ok(param.map_or(VVal::None, |param| param_id2vv(param)))
        }, Some(2), Some(2), false);

    st.fun(
        "inp_param", move |env: &mut Env, _argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            let param = env.arg(1).with_s_ref(|s| nid.inp_param(s));

            Ok(param.map_or(VVal::None, |param| param_id2vv(param)))
        }, Some(2), Some(2), false);

    st.fun(
        "param_list", move |env: &mut Env, _argc: usize| {
            let nid = vv2node_id(&env.arg(0));

            let atoms = VVal::vec();
            let mut i = 0;
            while let Some(param) = nid.atom_param_by_idx(i) {
                atoms.push(param_id2vv(param));
                i += 1;
            }

            let inputs = VVal::vec();
            let mut i = 0;
            while let Some(param) = nid.inp_param_by_idx(i) {
                inputs.push(param_id2vv(param));
                i += 1;
            }

            Ok(VVal::map2(
                "atoms",  atoms,
                "inputs", inputs,
            ))
        }, Some(1), Some(1), false);

    st.fun(
        "out_list", move |env: &mut Env, _argc: usize| {
            let nid = vv2node_id(&env.arg(0));

            let outputs = VVal::vec();
            let mut i = 0;
            while let Some(name) = nid.out_name_by_idx(i) {
                outputs.push(VVal::pair(
                    VVal::Int(i as i64),
                    VVal::new_str(name)));
                i += 1;
            }

            Ok(outputs)
        }, Some(1), Some(1), false);

    st.fun(
        "inp_name2idx", move |env: &mut Env, _argc: usize| {
            let nid   = vv2node_id(&env.arg(0));
            let idx = env.arg(1).with_s_ref(|s| nid.inp(s));
            Ok(idx.map_or(VVal::None, |idx| VVal::Int(idx as i64)))
        }, Some(2), Some(2), false);

    st.fun(
        "out_name2idx", move |env: &mut Env, _argc: usize| {
            let nid   = vv2node_id(&env.arg(0));
            let idx = env.arg(1).with_s_ref(|s| nid.out(s));
            Ok(idx.map_or(VVal::None, |idx| VVal::Int(idx as i64)))
        }, Some(2), Some(2), false);

    st.fun(
        "inp_idx2name", move |env: &mut Env, _argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            let name = nid.inp_name_by_idx(env.arg(1).i() as u8);
            Ok(name.map_or(VVal::None, |name| VVal::new_str(name)))
        }, Some(2), Some(2), false);

    st.fun(
        "out_idx2name", move |env: &mut Env, _argc: usize| {
            let nid  = vv2node_id(&env.arg(0));
            let name = nid.out_name_by_idx(env.arg(1).i() as u8);
            Ok(name.map_or(VVal::None, |name| VVal::new_str(name)))
        }, Some(2), Some(2), false);

    st
}

