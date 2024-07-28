use crate::{input::ffi, model::*, rewrites::*};
use egg::*;
// use cxx::UniquePtr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Wrapper class for egg's cost function
pub struct TensorCost<'a> {
    pub egraph: &'a EGraph<Mdl, TensorAnalysis>,
    pub cost_model: &'a CostModel<'a>,
}

impl CostFunction<Mdl> for TensorCost<'_> {
    type Cost = f32;
    /// Getting total cost for the subtree rooted at enode. See egg::CostFunction
    /// trait for more information on interface.
    fn cost<C: FnMut(Id) -> Self::Cost>(&mut self, enode: &Mdl, mut costs: C) -> Self::Cost {
        let self_cost = self.cost_model.get_self_cost(self.egraph, enode);
        enode.fold(self_cost, |sum, id| sum + costs(id))
    }
}

/// Class for our cost model
pub struct CostModel<'a> {
    cpp_cost_model: cxx::UniquePtr<ffi::CostModel>, // Holding the C++ cost model
    tensorinfo_map: &'a HashMap<Id, TensorInfo>,    // is this lifetime correct lol
}

impl<'a> CostModel<'a> {
    pub fn new(tensorinfo_map: &'a HashMap<Id, TensorInfo>) -> Self {
        Self {
            cpp_cost_model: ffi::newCostModel(),
            tensorinfo_map,
        }
    }

    pub fn tensor_data_to_shape_vec(&self, tensor_data: TensorData) -> Vec<i64> {
        tensor_data.shape.iter().map(|&x| x as i64).collect()
    }

    /// Gets cost for the enode itself.
    ///
    /// This function gets the cost by calling TASO's get_or_create_{some_op}()
    /// functions with the tensor information stored in metadata. TASO side stores
    /// hashmaps for OpBase objects. So here TASO side will simply lookup previously
    /// created ops (with previously measured runtime).
    ///
    /// # Parameters
    ///
    /// - `egraph`: E-graph of interest
    /// - `enode`: enode to get cost for
    ///
    /// # Returns
    ///
    /// Cost for this enode.
    pub fn get_self_cost(&self, egraph: &EGraph<Mdl, TensorAnalysis>, enode: &Mdl) -> f32 {
        let x = |i: &Id| &egraph[*i].data;
        match enode {
            Mdl::Num(_) | Mdl::Var(_) | Mdl::Input(_) => 0.0,
            Mdl::BlackBox(_) => 0.0,
            Mdl::CompareOp([input1, input2, comparison_direction, comparison_type]) => 0.0,
            Mdl::BroadcastInDimOp([input, broadcast_dimension]) => 0.0,
            Mdl::ConvertOp([input, output_type]) => 0.0,
            Mdl::ReduceOp([input, init_values]) => 0.0,
            Mdl::ReshapeOp([input, shape]) => 0.0,
            Mdl::GatherOp(
                [input, start_indices, offset_dims, collapsed_slice_dims, operand_batching_dims, start_indices_batching_dims, start_index_map, index_vector_dim, slice_sizes, indices_are_sorted],
            ) => 0.0,
            Mdl::SelectOp([pred, on_true, on_false]) => 0.0,
            Mdl::DotGeneralOp(
                [lhs, rhs, lhs_batch_dim, rhs_batch_dim, lhs_contract_dim, rhs_contract_dim, precision_config, shape],
            ) => 0.0,
            Mdl::PadOp(
                [input, padding_value, edge_padding_low, edge_padding_high, interior_padding],
            ) => 0.0,
            Mdl::SliceOp([input, start_indices, limit_indices, strides]) => 5.0,
            Mdl::TransposeOp([input, permutation]) => 3.0,
            Mdl::MulOp([lhs, rhs]) => {
                let lhs_dims = self.tensor_data_to_shape_vec(*x(lhs));
                let rhs_dims = self.tensor_data_to_shape_vec(*x(rhs));
                self.cpp_cost_model.getBinOpCost(
                    ffi::Ops::MulOp,
                    &lhs_dims,
                    ffi::Type::f32,
                    &rhs_dims,
                    ffi::Type::f32,
                ) as f32
            }
            Mdl::AddOp([lhs, rhs]) => {
                let lhs_dims = self.tensor_data_to_shape_vec(*x(lhs));
                let rhs_dims = self.tensor_data_to_shape_vec(*x(rhs));
                self.cpp_cost_model.getBinOpCost(
                    ffi::Ops::AddOp,
                    &lhs_dims,
                    ffi::Type::f32,
                    &rhs_dims,
                    ffi::Type::f32,
                ) as f32
            }
            Mdl::DivOp([lhs, rhs]) => {
                let lhs_dims = self.tensor_data_to_shape_vec(*x(lhs));
                let rhs_dims = self.tensor_data_to_shape_vec(*x(rhs));
                self.cpp_cost_model.getBinOpCost(
                    ffi::Ops::DivOp,
                    &lhs_dims,
                    ffi::Type::f32,
                    &rhs_dims,
                    ffi::Type::f32,
                ) as f32
            }
            Mdl::SubtractOp([lhs, rhs]) => {
                let lhs_dims = self.tensor_data_to_shape_vec(*x(lhs));
                let rhs_dims = self.tensor_data_to_shape_vec(*x(rhs));
                self.cpp_cost_model.getBinOpCost(
                    ffi::Ops::SubtractOp,
                    &lhs_dims,
                    ffi::Type::f32,
                    &rhs_dims,
                    ffi::Type::f32,
                ) as f32
            }
            Mdl::MinOp([lhs, rhs]) => {
                let lhs_dims = self.tensor_data_to_shape_vec(*x(lhs));
                let rhs_dims = self.tensor_data_to_shape_vec(*x(rhs));
                self.cpp_cost_model.getBinOpCost(
                    ffi::Ops::SubtractOp,
                    &lhs_dims,
                    ffi::Type::f32,
                    &rhs_dims,
                    ffi::Type::f32,
                ) as f32
            }

            Mdl::MaxOp([lhs, rhs]) => {
                let lhs_dims = self.tensor_data_to_shape_vec(*x(lhs));
                let rhs_dims = self.tensor_data_to_shape_vec(*x(rhs));
                self.cpp_cost_model.getBinOpCost(
                    ffi::Ops::SubtractOp,
                    &lhs_dims,
                    ffi::Type::f32,
                    &rhs_dims,
                    ffi::Type::f32,
                ) as f32
            }
            Mdl::NegOp([input]) => 5.0,
            Mdl::TanhOp([input]) => 9.0,
            Mdl::ExpOp([input]) => 5.0,
            Mdl::IotaOp([iota_dimension, shape]) => 5.0,
            Mdl::ConstantOp([]) => 1.0,
            Mdl::DynamicUpdateSliceOp([operand, update, start_indices]) => 3.0,
            Mdl::DynamicSliceOp([operand, start_indices, slice_sizes]) => 4.0,
            Mdl::ScatterOp([input, scatter_indices, updates, dimension_numbers]) => 6.0,
            _ => 0.0,
        }
    }
}

/// Prepare the data for formulation ILP
///
/// # Returns
///
/// - `m_id_map`: list of EClass Id's each index m refers to
/// - `e_m`: each entry is the list of nodes i within eclass m
/// - `h_i`: each entry is the list of children EClass indices for node i
/// - `cost_i`: self cost for each node i
/// - `g_i`: which EClass index does node i belong to
/// - `root_m`: EClass index of the root eclass
/// - `i_to_nodes: Vector of enodes, ordered by index i
/// - `blacklist_i: Vector of indices of nodes that are blacklisted
pub fn prep_ilp_data(
    egraph: &EGraph<Mdl, TensorAnalysis>,
    root: Id,
    cost_model: &CostModel,
) -> (
    Vec<Id>,
    Vec<Vec<usize>>,
    Vec<Vec<usize>>,
    Vec<f32>,
    Vec<usize>,
    usize,
    Vec<Mdl>,
    Vec<usize>,
) {
    let m_id_map: Vec<Id> = egraph.classes().map(|c| egraph.find(c.id)).collect();
    assert!(m_id_map.len() == egraph.number_of_classes());
    let id_m_map: HashMap<Id, usize> = m_id_map
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect();

    let num_classes = egraph.number_of_classes();
    let num_nodes = egraph.total_size();
    let mut i_to_nodes: Vec<Mdl> = Vec::with_capacity(num_nodes);
    let mut e_m: Vec<Vec<usize>> = vec![Vec::new(); num_classes];
    let mut h_i: Vec<Vec<usize>> = Vec::with_capacity(num_nodes);
    let mut cost_i: Vec<f32> = Vec::with_capacity(num_nodes);
    let mut g_i: Vec<usize> = Vec::with_capacity(num_nodes);
    let mut blacklist_i: Vec<usize> = Vec::new();

    let mut i = 0;
    for class in egraph.classes() {
        let m = *id_m_map.get(&egraph.find(class.id)).unwrap();
        for node in class.iter() {
            i_to_nodes.push(node.clone());
            if egraph.analysis.blacklist_nodes.contains(node) {
                blacklist_i.push(i);
            }
            e_m[m].push(i);
            h_i.push(
                node.children()
                    .iter()
                    .map(|id| *id_m_map.get(&egraph.find(*id)).unwrap())
                    .collect(),
            );
            println!("GETTING COST AS COST FOR {:?}", node);
            cost_i.push(cost_model.get_self_cost(egraph, node));
            println!("cost {:?}", cost_i.last());
            g_i.push(m);
            i += 1;
        }
    }

    let root_m = *id_m_map.get(&egraph.find(root)).unwrap();

    (
        m_id_map,
        e_m,
        h_i,
        cost_i,
        g_i,
        root_m,
        i_to_nodes,
        blacklist_i,
    )
}

/// Struct for storing the solved results from ILP
#[derive(Debug, Serialize, Deserialize)]
pub struct SolvedResults {
    /// The solved values for the variables associated with each node
    pub solved_x: Vec<i32>,
    /// The minimum total cost found
    pub cost: f32,
    /// Time for solver
    pub time: f32,
}

/// Construct the RecExpr of the optimized graph extracted
///
/// This function does the construction recursively with memoization. Call it with eclass=root
/// will construct the whole extracted graph
///
/// # Parameters
///
/// - `node_picked`: hashmap storing which node is picked for each EClass ID
/// - `eclass`: The EClass ID that we aim to construct as root
/// - `added_memo`: Map from EClass ID to RecExpr ID. Storing the eclasses that were already added
/// - `egraph`: E-graph of interest
/// - `expr`: the RecExpr storing the optimized graph, it is constructed within this function
///
/// # Returns
///
/// - The ID (index) in the output RecExpr for the eclass passed in as argument
pub fn construct_best_rec(
    node_picked: &HashMap<Id, Mdl>,
    eclass: Id,
    added_memo: &mut HashMap<Id, Id>,
    egraph: &EGraph<Mdl, TensorAnalysis>,
    expr: &mut RecExpr<Mdl>,
) -> Id {
    println!("infinitelooooop");
    let id = egraph.find(eclass);

    match added_memo.get(&id) {
        Some(id_expr) => *id_expr,
        None => {
            let node = node_picked.get(&id).unwrap().clone().map_children(|child| {
                construct_best_rec(node_picked, child, added_memo, egraph, expr)
            });
            let id_expr = expr.add(node);
            assert!(added_memo.insert(id, id_expr).is_none());
            id_expr
        }
    }
}

/// Get the initial solution for ILP using the greedy extraction
///
/// This function does the construction recursively with memoization. Call it with eclass=root
/// will construct the whole extracted graph
///
/// # Parameters
///
/// - `egraph`: egraph of interest
/// - `root`: root eclass
/// - `costs`: Map from eclass ID to the node with the lowest subtree cost (cost, node).
///         Constructed by egg's Extractor
/// - `g_i`: which EClass index does node i belong to
/// - `nodes_to_i`: map from node to index i
///
/// # Returns
///
/// A tuple of (i_list, m_list), where
///
/// - `i_list`: list of i picked by greedy extraction
/// - `m_list`: list of eclass index m that i_list belongs to
pub fn get_init_solution(
    egraph: &EGraph<Mdl, TensorAnalysis>,
    root: Id,
    costs: &HashMap<Id, (f32, Mdl)>,
    g_i: &[usize],
    nodes_to_i: &HashMap<Mdl, usize>,
) -> (Vec<usize>, Vec<usize>) {
    let mut nodes: Vec<Mdl> = Vec::new();
    // added_memo maps eclass id to id in expr
    let mut added_memo: HashSet<Id> = Default::default();
    get_init_rec(egraph, root, &mut added_memo, costs, &mut nodes);

    let i_list: Vec<usize> = nodes
        .iter()
        .map(|node| *nodes_to_i.get(node).unwrap())
        .collect();
    let m_list: Vec<usize> = i_list.iter().map(|i| g_i[*i]).collect();

    (i_list, m_list)
}

/// Recursively get the initial solution for ILP using the greedy extraction, results stored in nodes
///
/// # Parameters
///
/// - `egraph`: egraph of interest
/// - `eclass`: get solution rooted from here
/// - `added_memo`: Stores the set of eclasses that has already been processed
/// - `costs`: Map from eclass ID to the node with the lowest subtree cost (cost, node).
///         Constructed by egg's Extractor
/// - `nodes`: List of nodes picked by greedy extraction. Constructed within this function
fn get_init_rec(
    egraph: &EGraph<Mdl, TensorAnalysis>,
    eclass: Id,
    added_memo: &mut HashSet<Id>,
    costs: &HashMap<Id, (f32, Mdl)>,
    nodes: &mut Vec<Mdl>,
) {
    let id = egraph.find(eclass);

    if !added_memo.contains(&id) {
        let (_, best_node) = match costs.get(&id) {
            Some(result) => result.clone(),
            None => panic!("Failed to extract from eclass {}", id),
        };
        best_node.for_each(|child| get_init_rec(egraph, child, added_memo, costs, nodes));
        nodes.push(best_node);
        added_memo.insert(id);
    }
}
