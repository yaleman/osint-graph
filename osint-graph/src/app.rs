use std::{borrow::Cow, collections::HashMap};

use eframe::egui::{self, DragValue, TextStyle};
use egui_node_graph2::*;
use egui_notify::Toasts;
use serde::{Deserialize, Serialize};

use crate::storage::Backend;

// ========= First, define your user data types =============

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeData {
    template: NodeType,
    notes: String,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LinkType {
    Scalar,
    Vec2,
    Parent,
    Child,
    Related,
    // Link
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonData {
    name: String,
    aliases: Vec<String>,
}

/// In the graph, input parameters can optionally have a constant value. This
/// value can be directly edited in a widget inside the node itself.
///
/// There will usually be a correspondence between DataTypes and ValueTypes. But
/// this library makes no attempt to check this consistency. For instance, it is
/// up to the user code in this example to make sure no parameter is created
/// with a DataType of Scalar and a ValueType of Vec2.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ValueType {
    Vec2 { value: egui::Vec2 },
    Scalar { value: f32 },
    Person { value: PersonData },
    Weak,
}

impl Default for ValueType {
    fn default() -> Self {
        // NOTE: This is just a dummy `Default` implementation. The library
        // requires it to circumvent some internal borrow checker issues.
        // Self::Scalar { value: 0.0 }
        Self::Weak
    }
}

impl ValueType {
    /// Tries to downcast this value type to a vector
    pub fn try_to_vec2(self) -> anyhow::Result<egui::Vec2> {
        if let ValueType::Vec2 { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to vec2", self)
        }
    }

    /// Tries to downcast this value type to a scalar
    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        if let ValueType::Scalar { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to scalar", self)
        }
    }
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, enum_iterator::Sequence)]
pub enum NodeType {
    MakeScalar,
    AddScalar,
    SubtractScalar,
    MakeVector,
    AddVector,
    SubtractVector,
    VectorTimesScalar,
    GenericURL,
    File,
    Image,
    Audio,
    Video,
    Person,
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyResponse {
    SetActiveNode(NodeId),
    ClearActiveNode,
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct MyGraphState {
    pub active_node: Option<NodeId>,
}

// =========== Then, you need to implement some traits ============

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<MyGraphState> for LinkType {
    fn data_type_color(&self, _user_state: &mut MyGraphState) -> egui::Color32 {
        match self {
            LinkType::Scalar => egui::Color32::from_rgb(38, 109, 211),
            LinkType::Vec2 => egui::Color32::from_rgb(238, 207, 109),
            LinkType::Parent => egui::Color32::DARK_RED,
            LinkType::Child => egui::Color32::LIGHT_BLUE,
            LinkType::Related => egui::Color32::DARK_RED,
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            LinkType::Scalar => Cow::Borrowed("scalar"),
            LinkType::Vec2 => Cow::Borrowed("2d vector"),
            LinkType::Parent => Cow::Borrowed("parent"),
            LinkType::Child => Cow::Borrowed("child"),
            LinkType::Related => Cow::Borrowed("related"),
        }
    }
}

pub enum NodeCategory {
    Scalar,
    Vector,
    URLs,
    File,
    People,
}

impl CategoryTrait for NodeCategory {
    fn name(&self) -> String {
        match self {
            NodeCategory::Scalar => "Scalar",
            NodeCategory::Vector => "Vector",
            NodeCategory::URLs => "URLs",
            NodeCategory::File => "Files",
            NodeCategory::People => "People",
        }
        .to_string()
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for NodeType {
    type NodeData = NodeData;
    type DataType = LinkType;
    type ValueType = ValueType;
    type UserState = MyGraphState;
    type CategoryType = NodeCategory;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        Cow::Borrowed(match self {
            NodeType::MakeScalar => "New scalar",
            NodeType::AddScalar => "Scalar add",
            NodeType::SubtractScalar => "Scalar subtract",
            NodeType::MakeVector => "New vector",
            NodeType::AddVector => "Vector add",
            NodeType::SubtractVector => "Vector subtract",
            NodeType::VectorTimesScalar => "Vector times scalar",
            NodeType::GenericURL => "Generic URL",
            NodeType::File => "File",
            NodeType::Image => "Image",
            NodeType::Audio => "Audio",
            NodeType::Video => "Video",
            NodeType::Person => "Person",
        })
    }

    // this is what allows the library to show collapsible lists in the node finder.
    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<NodeCategory> {
        match self {
            NodeType::MakeScalar | NodeType::AddScalar | NodeType::SubtractScalar => {
                vec![NodeCategory::Scalar]
            }
            NodeType::MakeVector | NodeType::AddVector | NodeType::SubtractVector => {
                vec![NodeCategory::Vector]
            }
            NodeType::GenericURL => vec![NodeCategory::URLs],

            NodeType::VectorTimesScalar => vec![NodeCategory::Vector, NodeCategory::Scalar],
            NodeType::Image | NodeType::File | NodeType::Audio | NodeType::Video => {
                vec![NodeCategory::File]
            }
            NodeType::Person => vec![NodeCategory::People],
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        NodeData {
            template: *self,
            notes: String::new(),
        }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        // The nodes are created empty by default. This function needs to take
        // care of creating the desired inputs and outputs based on the template

        // We define some closures here to avoid boilerplate. Note that this is
        // entirely optional.
        let input_scalar = |graph: &mut MyGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                LinkType::Scalar,
                ValueType::Scalar { value: 0.0 },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_vector = |graph: &mut MyGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                LinkType::Vec2,
                ValueType::Vec2 {
                    value: egui::vec2(0.0, 0.0),
                },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };

        // let input_parent = |graph: &mut MyGraph, name: &str| {
        //     graph.add_input_param(
        //         node_id,
        //         name.to_string(),
        //         LinkType::Parent,
        //         ValueType::Weak,
        //         InputParamKind::ConnectionOnly,
        //         true,
        //     );
        // };

        let output_scalar = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), LinkType::Scalar);
        };
        let output_vector = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), LinkType::Vec2);
        };
        let output_child = |graph: &mut MyGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), LinkType::Parent);
        };

        match self {
            NodeType::Person => {
                graph.add_input_param(
                    node_id,
                    "".to_string(),
                    LinkType::Parent,
                    ValueType::Weak,
                    InputParamKind::ConnectionOnly,
                    true,
                );
                // input_parent(graph, "parent");
                output_child(graph, "Child");
            }
            NodeType::Image => todo!(),
            NodeType::File => todo!(),
            NodeType::Audio => todo!(),
            NodeType::Video => todo!(),
            NodeType::GenericURL => todo!(),
            NodeType::AddScalar => {
                // The first input param doesn't use the closure so we can comment
                // it in more detail.
                graph.add_input_param(
                    node_id,
                    // This is the name of the parameter. Can be later used to
                    // retrieve the value. Parameter names should be unique.
                    "A".into(),
                    // The data type for this input. In this case, a scalar
                    LinkType::Scalar,
                    // The value type for this input. We store zero as default
                    ValueType::Scalar { value: 0.0 },
                    // The input parameter kind. This allows defining whether a
                    // parameter accepts input connections and/or an inline
                    // widget to set its value.
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
                input_scalar(graph, "B");
                output_scalar(graph, "out");
            }
            NodeType::SubtractScalar => {
                input_scalar(graph, "A");
                input_scalar(graph, "B");
                output_scalar(graph, "out");
            }
            NodeType::VectorTimesScalar => {
                input_scalar(graph, "scalar");
                input_vector(graph, "vector");
                output_vector(graph, "out");
            }
            NodeType::AddVector => {
                input_vector(graph, "v1");
                input_vector(graph, "v2");
                output_vector(graph, "out");
            }
            NodeType::SubtractVector => {
                input_vector(graph, "v1");
                input_vector(graph, "v2");
                output_vector(graph, "out");
            }
            NodeType::MakeVector => {
                input_scalar(graph, "x");
                input_scalar(graph, "y");
                output_vector(graph, "out");
            }
            NodeType::MakeScalar => {
                input_scalar(graph, "value");
                output_scalar(graph, "out");
            }
        }
    }
}

pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = NodeType;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // This function must return a list of node kinds, which the node finder
        // will use to display it to the user. Crates like strum can reduce the
        // boilerplate in enumerating all variants of an enum.
        enum_iterator::all::<Self::Item>().collect::<Vec<_>>()
    }
}

impl WidgetValueTrait for ValueType {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type NodeData = NodeData;
    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut MyGraphState,
        _node_data: &NodeData,
    ) -> Vec<MyResponse> {
        // This trait is used to tell the library which UI to display for the
        // inline parameter widgets.
        match self {
            ValueType::Person { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::TextEdit::singleline(&mut value.name).hint_text("Name"));
                });
            }
            ValueType::Weak => {
                ui.label(param_name);
                // ui.label("");
            }
            ValueType::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut value.x));
                    ui.label("y");
                    ui.add(DragValue::new(&mut value.y));
                });
            }
            ValueType::Scalar { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
        }
        // This allows you to return your responses from the inline widgets.
        Vec::new()
    }
}

impl UserResponseTrait for MyResponse {}
impl NodeDataTrait for NodeData {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type DataType = LinkType;
    type ValueType = ValueType;

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes. In this case, we create an "active"
    // button which introduces the concept of having an active node in the
    // graph. This is done entirely from user code with no modifications to the
    // node graph library.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        _graph: &Graph<NodeData, LinkType, ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<MyResponse, NodeData>>
    where
        MyResponse: UserResponseTrait,
    {
        // This logic is entirely up to the user. In this case, we check if the
        // current node we're drawing is the active one, by comparing against
        // the value stored in the global user state, and draw different button
        // UIs based on that.

        let mut responses = vec![];
        let is_active = user_state
            .active_node
            .map(|id| id == node_id)
            .unwrap_or(false);

        // Pressing the button will emit a custom user response to either set,
        // or clear the active node. These responses do nothing by themselves,
        // the library only makes the responses available to you after the graph
        // has been drawn. See below at the update method for an example.
        if !is_active {
            if ui.button("👁 Set active").clicked() {
                responses.push(NodeResponse::User(MyResponse::SetActiveNode(node_id)));
            }
        } else {
            let button =
                egui::Button::new(egui::RichText::new("👁 Active").color(egui::Color32::BLACK))
                    .fill(egui::Color32::GOLD);
            if ui.add(button).clicked() {
                responses.push(NodeResponse::User(MyResponse::ClearActiveNode));
            }
        }

        responses
    }
}

type MyGraph = Graph<NodeData, LinkType, ValueType>;
type MyEditorState = GraphEditorState<NodeData, LinkType, ValueType, NodeType, MyGraphState>;

#[derive(Default)]
pub struct OsintGraph {
    // The `GraphEditorState` is the top-level object. You "register" all your
    // custom types by specifying it as its generic parameters.
    state: MyEditorState,

    user_state: MyGraphState,

    #[allow(dead_code)]
    messages: Toasts,

    #[allow(dead_code)]
    storage: crate::storage::Backend,
}

const PERSISTENCE_KEY: &str = "osint-graph";

impl OsintGraph {
    /// If the persistence feature is enabled, Called once before the first frame.
    /// Load previous app state (if any).
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, PERSISTENCE_KEY))
            .unwrap_or_default();
        Self {
            state,
            user_state: MyGraphState::default(),
            messages: Toasts::default(),
            storage: Backend::default(),
        }
    }
}

impl eframe::App for OsintGraph {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, PERSISTENCE_KEY, &self.state);

        // self.storage
        //     .set(PERSISTENCE_KEY, serde_json::to_string(&self.state).unwrap());
    }
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
            });
        });
        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.state.draw_graph_editor(
                    ui,
                    AllMyNodeTemplates,
                    &mut self.user_state,
                    Vec::default(),
                )
            })
            .inner;
        for node_response in graph_response.node_responses {
            // Here, we ignore all other graph events. But you may find
            // some use for them. For example, by playing a sound when a new
            // connection is created
            if let NodeResponse::User(user_event) = node_response {
                match user_event {
                    MyResponse::SetActiveNode(node) => self.user_state.active_node = Some(node),
                    MyResponse::ClearActiveNode => self.user_state.active_node = None,
                }
            }
        }

        if let Some(node) = self.user_state.active_node {
            if self.state.graph.nodes.contains_key(node) {
                let text = match evaluate_node(&self.state.graph, node, &mut HashMap::new()) {
                    Ok(value) => format!("The result is: {:?}", value),
                    Err(err) => format!("Execution error: {}", err),
                };
                ctx.debug_painter().text(
                    egui::pos2(10.0, 35.0),
                    egui::Align2::LEFT_TOP,
                    text,
                    TextStyle::Button.resolve(&ctx.style()),
                    egui::Color32::WHITE,
                );
            } else {
                self.user_state.active_node = None;
            }
        }
    }
}

type OutputsCache = HashMap<OutputId, ValueType>;

/// Recursively evaluates all dependencies of this node, then evaluates the node itself.
pub fn evaluate_node(
    graph: &MyGraph,
    node_id: NodeId,
    outputs_cache: &mut OutputsCache,
) -> anyhow::Result<ValueType> {
    // To solve a similar problem as creating node types above, we define an
    // Evaluator as a convenience. It may be overkill for this small example,
    // but something like this makes the code much more readable when the
    // number of nodes starts growing.

    struct Evaluator<'a> {
        graph: &'a MyGraph,
        outputs_cache: &'a mut OutputsCache,
        node_id: NodeId,
    }
    impl<'a> Evaluator<'a> {
        fn new(graph: &'a MyGraph, outputs_cache: &'a mut OutputsCache, node_id: NodeId) -> Self {
            Self {
                graph,
                outputs_cache,
                node_id,
            }
        }
        fn evaluate_input(&mut self, name: &str) -> anyhow::Result<ValueType> {
            // Calling `evaluate_input` recursively evaluates other nodes in the
            // graph until the input value for a paramater has been computed.
            evaluate_input(self.graph, self.node_id, name, self.outputs_cache)
        }
        fn populate_output(&mut self, name: &str, value: ValueType) -> anyhow::Result<ValueType> {
            // After computing an output, we don't just return it, but we also
            // populate the outputs cache with it. This ensures the evaluation
            // only ever computes an output once.
            //
            // The return value of the function is the "final" output of the
            // node, the thing we want to get from the evaluation. The example
            // would be slightly more contrived when we had multiple output
            // values, as we would need to choose which of the outputs is the
            // one we want to return. Other outputs could be used as
            // intermediate values.
            //
            // Note that this is just one possible semantic interpretation of
            // the graphs, you can come up with your own evaluation semantics!
            populate_output(self.graph, self.outputs_cache, self.node_id, name, value)
        }
        fn input_vector(&mut self, name: &str) -> anyhow::Result<egui::Vec2> {
            self.evaluate_input(name)?.try_to_vec2()
        }
        fn input_scalar(&mut self, name: &str) -> anyhow::Result<f32> {
            self.evaluate_input(name)?.try_to_scalar()
        }
        fn output_vector(&mut self, name: &str, value: egui::Vec2) -> anyhow::Result<ValueType> {
            self.populate_output(name, ValueType::Vec2 { value })
        }
        fn output_scalar(&mut self, name: &str, value: f32) -> anyhow::Result<ValueType> {
            self.populate_output(name, ValueType::Scalar { value })
        }
    }

    let node = &graph[node_id];
    let mut evaluator = Evaluator::new(graph, outputs_cache, node_id);
    match node.user_data.template {
        NodeType::Person => Err(anyhow::anyhow!("Not implemented")),
        NodeType::File => todo!(),
        NodeType::Image => todo!(),
        NodeType::Audio => todo!(),
        NodeType::Video => todo!(),
        NodeType::GenericURL => todo!(),
        NodeType::AddScalar => {
            let a = evaluator.input_scalar("A")?;
            let b = evaluator.input_scalar("B")?;
            evaluator.output_scalar("out", a + b)
        }
        NodeType::SubtractScalar => {
            let a = evaluator.input_scalar("A")?;
            let b = evaluator.input_scalar("B")?;
            evaluator.output_scalar("out", a - b)
        }
        NodeType::VectorTimesScalar => {
            let scalar = evaluator.input_scalar("scalar")?;
            let vector = evaluator.input_vector("vector")?;
            evaluator.output_vector("out", vector * scalar)
        }
        NodeType::AddVector => {
            let v1 = evaluator.input_vector("v1")?;
            let v2 = evaluator.input_vector("v2")?;
            evaluator.output_vector("out", v1 + v2)
        }
        NodeType::SubtractVector => {
            let v1 = evaluator.input_vector("v1")?;
            let v2 = evaluator.input_vector("v2")?;
            evaluator.output_vector("out", v1 - v2)
        }
        NodeType::MakeVector => {
            let x = evaluator.input_scalar("x")?;
            let y = evaluator.input_scalar("y")?;
            evaluator.output_vector("out", egui::vec2(x, y))
        }
        NodeType::MakeScalar => {
            let value = evaluator.input_scalar("value")?;
            evaluator.output_scalar("out", value)
        }
    }
}

fn populate_output(
    graph: &MyGraph,
    outputs_cache: &mut OutputsCache,
    node_id: NodeId,
    param_name: &str,
    value: ValueType,
) -> anyhow::Result<ValueType> {
    let output_id = graph[node_id].get_output(param_name)?;
    outputs_cache.insert(output_id, value.clone());
    Ok(value)
}

// Evaluates the input value of
fn evaluate_input(
    graph: &MyGraph,
    node_id: NodeId,
    param_name: &str,
    outputs_cache: &mut OutputsCache,
) -> anyhow::Result<ValueType> {
    let input_id = graph[node_id].get_input(param_name)?;

    // The output of another node is connected.
    if let Some(other_output_id) = graph.connection(input_id) {
        // The value was already computed due to the evaluation of some other
        // node. We simply return value from the cache.
        if let Some(other_value) = outputs_cache.get(&other_output_id) {
            Ok(other_value.clone())
        }
        // This is the first time encountering this node, so we need to
        // recursively evaluate it.
        else {
            // Calling this will populate the cache
            evaluate_node(graph, graph[other_output_id].node, outputs_cache)?;

            // Now that we know the value is cached, return it
            Ok(outputs_cache
                .get(&other_output_id)
                .expect("Cache should be populated")
                .clone())
        }
    }
    // No existing connection, take the inline value instead.
    else {
        Ok(graph[input_id].value.clone())
    }
}
