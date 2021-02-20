const MAX_ALLOCATED_NODES : usize = 256;
const MAX_NODE_PROG_OPS   : usize = 256 * 3;

use ringbuf::{RingBuffer, Producer, Consumer};
use crate::dsp::{node_factory, NodeInfo, Node, NodeId};

#[derive(Debug, Clone)]
pub struct NodeProg {
    pub out: Vec<f32>,
    pub prog: Vec<NodeOp>,
}

impl NodeProg {
    pub fn empty() -> Self {
        Self {
            out: vec![],
            prog: vec![],
        }
    }

    pub fn new(out_len: usize) -> Self {
        let mut out = vec![];
        out.resize(out_len, 0.0);
        Self {
            out,
            prog: vec![],
        }
    }

    pub fn append_with_inputs(
        &mut self,
        node_op: NodeOp,
        inp1: Option<(usize, usize)>,
        inp2: Option<(usize, usize)>,
        inp3: Option<(usize, usize)>)
    {
        let mut index = Option<usize>;

        for n_op in self.prog.iter_mut() {
            if n_op.idx == node_op.idx {
                if let Some(inp1) = inp1 {
//                    n_op.inputs.push(
                }
            }
        }
    }
}

/// Big messages for updating the NodeExecutor thread.
/// Usually used for shoveling NodeProg and Nodes to and from
/// the NodeExecutor thread.
#[derive(Debug, Clone)]
pub enum GraphMessage {
    NewNode { index: u8, node: Node },
    NewProg { prog: NodeProg },
}

/// Messages for small updates between the NodeExecutor thread
/// and the NodeConfigurator.
pub enum QuickMessage {
    ParamUpdate { node_id: u8, param_id: u8, value: f32 },
    Feedback    { node_id: u8, feedback_id: u8, value: f32 },
}

/// For receiving deleted/overwritten nodes from the backend
/// thread and dropping them.
struct DropThread {
    terminate: std::sync::Arc<std::sync::atomic::AtomicBool>,
    th:        Option<std::thread::JoinHandle<()>>,
}

impl DropThread {
    fn new(mut graph_drop_con: Consumer<DropMsg>) -> Self {
        let terminate =
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let th_terminate = terminate.clone();

        let th = std::thread::spawn(move || {
            loop {
                if th_terminate.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                while let Some(_node) = graph_drop_con.pop() {
                    // drop it ...
                    println!("Dropped some shit...");
                }

                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        });

        Self {
            th: Some(th),
            terminate,
        }
    }
}

impl Drop for DropThread {
    fn drop(&mut self) {
        self.terminate.store(true, std::sync::atomic::Ordering::Relaxed);
        self.th.take().unwrap().join();
    }
}

/// This struct holds the frontend node configuration.
///
/// It stores which nodes are allocated and where.
/// Allocation of new nodes is done here, and parameter management
/// and synchronization is also done by this. It generally acts
/// as facade for the executed node graph in the backend.
pub struct NodeConfigurator {
    /// Holds all the nodes, their parameters and type.
    nodes:              Vec<NodeInfo>,
    /// For updating the NodeExecutor with graph updates.
    graph_update_prod:  Producer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    quick_update_prod:  Producer<QuickMessage>,
    /// For receiving feedback from the backend thread.
    feedback_con:       Consumer<QuickMessage>,
    /// Handles deallocation
    drop_thread:        DropThread,
    sample_rate:        f32,
}

impl NodeConfigurator {
    pub fn drop_node(&mut self, idx: usize) {
        if idx >= self.nodes.len() {
            return;
        }

        match self.nodes[idx] {
            NodeInfo::Nop => { return; },
            _ => {},
        }

        self.nodes[idx] = NodeInfo::Nop;
        self.graph_update_prod.push(
            GraphMessage::NewNode {
                index: idx as u8,
                node: Node::Nop,
            });
    }

    pub fn for_each<F: FnMut(&NodeInfo, NodeId, usize)>(&self, mut f: F) {
        let mut i = 0;
        for n in self.nodes.iter() {
            let nid = n.to_id(0);
            if NodeId::Nop == nid {
                break;
            }

            f(n, nid, i);
            i += 1;
        }
    }

    pub fn create_node(&mut self, ni: NodeId) -> Option<u8> {
        if let Some((node, info)) = node_factory(ni, self.sample_rate) {
            let mut index : Option<usize> = None;
            for i in 0..self.nodes.len() {
                if let NodeInfo::Nop = self.nodes[i] {
                    index = Some(i);
                    break;
                }
            }

            if let Some(index) = index {
                self.nodes[index] = info;
                self.graph_update_prod.push(
                    GraphMessage::NewNode { index: index as u8, node });
                Some(index as u8)
            } else {
                let index = self.nodes.len();
                self.nodes.resize_with((self.nodes.len() + 1) * 2, || NodeInfo::Nop);
                self.nodes[index] = info;
                self.graph_update_prod.push(
                    GraphMessage::NewNode { index: index as u8, node });
                Some(index as u8)
            }
        } else {
            None
        }
    }

    pub fn upload_prog(&mut self, prog: NodeProg) {
        self.graph_update_prod.push(GraphMessage::NewProg { prog });
    }
}

/// Creates a NodeConfigurator and a NodeExecutor which are interconnected
/// by ring buffers.
pub fn new_node_engine(sample_rate: f32) -> (NodeConfigurator, NodeExecutor) {
    let rb_graph     = RingBuffer::new(MAX_ALLOCATED_NODES * 2);
    let rb_quick     = RingBuffer::new(MAX_ALLOCATED_NODES * 8);
    let rb_drop      = RingBuffer::new(MAX_ALLOCATED_NODES * 2);
    let rb_feedback  = RingBuffer::new(MAX_ALLOCATED_NODES);

    let (rb_graph_prod, rb_graph_con) = rb_graph.split();
    let (rb_quick_prod, rb_quick_con) = rb_quick.split();
    let (rb_drop_prod,  rb_drop_con)  = rb_drop.split();
    let (rb_fb_prod,    rb_fb_con)    = rb_feedback.split();

    let drop_thread = DropThread::new(rb_drop_con);

    let mut nodes = Vec::new();
    nodes.resize_with(MAX_ALLOCATED_NODES, || NodeInfo::Nop);

    let nc = NodeConfigurator {
        nodes,
        graph_update_prod: rb_graph_prod,
        quick_update_prod: rb_quick_prod,
        feedback_con:      rb_fb_con,
        drop_thread,
        sample_rate,
    };

    let mut nodes = Vec::new();
    nodes.resize_with(MAX_ALLOCATED_NODES, || Node::Nop);

    let ne = NodeExecutor {
        sample_rate,
        nodes,
        prog:              NodeProg::empty(),
        graph_update_con:  rb_graph_con,
        quick_update_con:  rb_quick_con,
        graph_drop_prod:   rb_drop_prod,
        feedback_prod:     rb_fb_prod,
    };

    (nc, ne)
}

/// Operator for transmitting the output of a node
/// to the input of another node.
#[derive(Debug, Clone, Copy)]
pub struct OutOp {
    pub out_port_idx:  u8,
    pub node_idx:      u8,
    pub dst_param_idx: u8
}

/// Step in a `NodeProg` that stores the to be
/// executed node and output operations.
#[derive(Debug, Clone)]
pub struct NodeOp {
    /// Stores the index of the node
    pub idx:  u8,
    /// Output index and length of the node:
    pub out_idxlen: (usize, usize),
    /// Input indices, (<out vec index>, <own node input index>)
    pub inputs: Vec<(usize, usize)>,
}

impl NodeOp {
    fn empty() -> Self {
        Self {
            idx:        0,
            out_idxlen: (0, 0),
            inputs:     vec![],
        }
    }
}

#[derive(Debug)]
enum DropMsg {
    Node { node: Node },
    Prog { prog: NodeProg },
}

/// Holds the complete allocation of nodes and
/// the program. New Nodes or the program is
/// not newly allocated in the audio backend, but it is
/// copied from the input ring buffer.
/// If this turns out to be too slow, we might
/// have to push buffers of the program around.
///
pub struct NodeExecutor {
    /// Contains the nodes and their state.
    /// Is loaded from the input ring buffer when a corresponding
    /// message arrives.
    ///
    /// In case the previous node contained something that needs
    /// deallocation, the nodes are replaced and the contents
    /// is sent back using the free-ringbuffer.
    nodes: Vec<Node>,

    /// Contains the to be executed nodes and output operations.
    /// Is copied from the input ringbuffer when a corresponding
    /// message arrives.
    prog: NodeProg,

    /// For receiving Node and NodeProg updates
    graph_update_con:  Consumer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    quick_update_con:  Consumer<QuickMessage>,
    /// For receiving deleted/overwritten nodes from the backend thread.
    graph_drop_prod:   Producer<DropMsg>,
    /// For receiving feedback from the backend thread.
    feedback_prod:     Producer<QuickMessage>,

    /// The sample rate
    sample_rate: f32,
}

pub trait NodeAudioContext {
    fn output(&mut self, channel: usize, v: f32);
    fn input(&mut self, channel: usize) -> f32;
}

impl NodeExecutor {
    #[inline]
    pub fn process_graph_updates(&mut self) {
        while let Some(upd) = self.graph_update_con.pop() {
            println!("UPDATE GRAPH {:?}", upd);
            match upd {
                GraphMessage::NewNode { index, node } => {
                    let prev_node =
                        std::mem::replace(
                            &mut self.nodes[index as usize],
                            node);
                    self.graph_drop_prod.push(DropMsg::Node { node: prev_node });
                },
                GraphMessage::NewProg { prog } => {
                    let prev_prog =
                        std::mem::replace(
                            &mut self.prog,
                            prog);
                    self.graph_drop_prod.push(DropMsg::Prog { prog: prev_prog });
                },
            }
        }

        // TODO: Handle quick_update_con to start ramps for the
        //       passed parameters.
    }

    #[inline]
    pub fn get_nodes(&self) -> &Vec<Node> { &self.nodes }

    #[inline]
    pub fn get_prog(&self) -> &NodeProg { &self.prog }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T) {
        let nodes = &mut self.nodes;
        for op in self.prog.prog.iter() {
            nodes[op.idx as usize]
            .process(ctx, &op.inputs, &op.out_idxlen, &mut self.prog.out);
        }
    }
}
