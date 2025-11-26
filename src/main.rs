use eframe::egui;
use egui_node_graph2::*;
use std::borrow::Cow;

// --- Data Types ---

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MyDataType {
    Midi,
    Audio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyValueType {
    None,
}

impl Default for MyValueType {
    fn default() -> Self {
        MyValueType::None
    }
}

#[derive(Clone, Debug)]
pub struct MyNodeData {
    title: String,
}

// --- Graph State ---

#[derive(Default)]
pub struct MyGraphState;

impl DataTypeTrait<MyGraphState> for MyDataType {
    fn data_type_color(&self, _user_state: &mut MyGraphState) -> egui::Color32 {
        match self {
            MyDataType::Midi => egui::Color32::from_rgb(255, 215, 0),   // Gold
            MyDataType::Audio => egui::Color32::from_rgb(0, 191, 255), // DeepSkyBlue
        }
    }

    fn name(&self) -> Cow<str> {
        match self {
            MyDataType::Midi => Cow::Borrowed("MIDI"),
            MyDataType::Audio => Cow::Borrowed("Audio"),
        }
    }
}

impl NodeDataTrait for MyNodeData {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type DataType = MyDataType;
    type ValueType = MyValueType;

    fn can_delete(
        &self,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> bool {
        true
    }

    fn bottom_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<MyResponse, MyNodeData>>
    where
        MyResponse: UserResponseTrait,
    {
        Vec::new()
    }
}

impl WidgetValueTrait for MyValueType {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type NodeData = MyNodeData;

    fn value_widget(
        &mut self,
        _param_name: &str,
        _node_id: NodeId,
        _ui: &mut egui::Ui,
        _user_state: &mut MyGraphState,
        _node_data: &MyNodeData,
    ) -> Vec<MyResponse> {
        Vec::new()
    }
}

#[derive(Clone, Debug)]
pub enum MyResponse {
    // No custom responses needed for UI prototype
}

impl UserResponseTrait for MyResponse {}

type MyGraph = Graph<MyNodeData, MyDataType, MyValueType>;
type MyEditorState = GraphEditorState<MyNodeData, MyDataType, MyValueType, MyNodeTemplate, MyGraphState>;

// --- Node Templates (for creation) ---

#[derive(Clone, Copy, Debug)]
pub enum MyNodeTemplate {
    MidiNode,
    SoundSource,
    Effect,
    Mixer,
}

impl NodeTemplateTrait for MyNodeTemplate {
    type NodeData = MyNodeData;
    type DataType = MyDataType;
    type ValueType = MyValueType;
    type UserState = MyGraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<str> {
        match self {
            MyNodeTemplate::MidiNode => Cow::Borrowed("MIDI Input"),
            MyNodeTemplate::SoundSource => Cow::Borrowed("Sound Source"),
            MyNodeTemplate::Effect => Cow::Borrowed("Audio Effect"),
            MyNodeTemplate::Mixer => Cow::Borrowed("Audio Mixer"),
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).to_string()
    }

    fn user_data(&self, user_state: &mut Self::UserState) -> Self::NodeData {
        MyNodeData {
            title: self.node_finder_label(user_state).to_string(),
        }
    }

    fn build_node(
        &self,
        graph: &mut MyGraph,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        match self {
            MyNodeTemplate::MidiNode => {
                graph.add_output_param(node_id, "MIDI Out".to_string(), MyDataType::Midi);
            }
            MyNodeTemplate::SoundSource => {
                graph.add_input_param(
                    node_id,
                    "MIDI In".to_string(),
                    MyDataType::Midi,
                    MyValueType::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_output_param(node_id, "Audio Out".to_string(), MyDataType::Audio);
            }
            MyNodeTemplate::Effect => {
                graph.add_input_param(
                    node_id,
                    "Audio In".to_string(),
                    MyDataType::Audio,
                    MyValueType::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_output_param(node_id, "Audio Out".to_string(), MyDataType::Audio);
            }
            MyNodeTemplate::Mixer => {
                graph.add_input_param(
                    node_id,
                    "In 1".to_string(),
                    MyDataType::Audio,
                    MyValueType::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "In 2".to_string(),
                    MyDataType::Audio,
                    MyValueType::None,
                    InputParamKind::ConnectionOnly,
                    true,
                );
                graph.add_output_param(node_id, "Mix Out".to_string(), MyDataType::Audio);
            }
        }
    }
}

// --- Main App ---

struct HarmoniaApp {
    state: MyEditorState,
    user_state: MyGraphState,
    context_menu_pos: Option<egui::Pos2>,
}

impl HarmoniaApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: GraphEditorState::new(1.0),
            user_state: MyGraphState::default(),
            context_menu_pos: None,
        }
    }

    fn draw_grid(&self, ui: &egui::Ui) {
        let rect = ui.max_rect();
        let zoom = self.state.pan_zoom.zoom;
        let pan = self.state.pan_zoom.pan;
        let grid_size = 20.0; 

        // Don't draw if too small to see/performant
        if grid_size * zoom < 4.0 {
            return;
        }

        let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15); // Faint white dots

        let painter = ui.painter();
        
        // Calculate visible bounds in grid coordinates
        let start_col = ((rect.min.x - pan.x) / (grid_size * zoom)).floor() as i64;
        let end_col = ((rect.max.x - pan.x) / (grid_size * zoom)).ceil() as i64;
        let start_row = ((rect.min.y - pan.y) / (grid_size * zoom)).floor() as i64;
        let end_row = ((rect.max.y - pan.y) / (grid_size * zoom)).ceil() as i64;

        // Safety check
        let cols = end_col - start_col;
        let rows = end_row - start_row;
        if cols * rows > 100_000 {
            return; 
        }

        for c in start_col..=end_col {
            let x = pan.x + c as f32 * grid_size * zoom;
            for r in start_row..=end_row {
                let y = pan.y + r as f32 * grid_size * zoom;
                let pos = egui::pos2(x, y);
                if rect.contains(pos) {
                    painter.circle_filled(pos, 1.0, color);
                }
            }
        }
    }
}

impl eframe::App for HarmoniaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {}
                    if ui.button("Open Project").clicked() {}
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {}
                    if ui.button("Redo").clicked() {}
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Zoom In").clicked() {}
                    if ui.button("Zoom Out").clicked() {}
                });
            });
        });

        // 2. Bottom Info Bar
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Harmonia Engine Status: Ready");
                ui.separator();
                ui.label(format!("Nodes: {}", self.state.graph.nodes.len()));
            });
        });

        // 3. Central Panel (Graph Editor)
        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| {
                // Custom Zoom Logic: Mouse-centered zoom
                // We intercept the scroll input and manually adjust pan and zoom.
                let rect = ui.max_rect();
                
                // Only zoom if mouse is over the area
                if ui.rect_contains_pointer(rect) {
                     let (scroll_delta, mouse_pos) = ui.input(|i| {
                        // Combine smooth and raw scroll (mouse wheel usually gives raw or smooth depending on OS/config)
                        let delta = i.smooth_scroll_delta.y + i.raw_scroll_delta.y; 
                        (delta, i.pointer.hover_pos())
                    });

                    if scroll_delta != 0.0 {
                        if let Some(mouse_pos) = mouse_pos {
                             // Calculate zoom factor
                             // Scaling: 0.1% per unit of scroll? Adjust sensitivity as needed.
                             let zoom_factor = (scroll_delta * 0.002).exp(); 
                             
                             let old_zoom = self.state.pan_zoom.zoom;
                             let old_pan = self.state.pan_zoom.pan;
                             
                             let new_zoom = old_zoom * zoom_factor;
                             
                             // Calculate new pan to keep mouse_pos fixed relative to graph
                             // Formula: P_new = M - (M - P_old) * (Z_new / Z_old)
                             let mouse_vec = mouse_pos.to_vec2();
                             let new_pan = mouse_vec - (mouse_vec - old_pan) * (new_zoom / old_zoom);
                             
                             self.state.pan_zoom.zoom = new_zoom;
                             self.state.pan_zoom.pan = new_pan;
                             
                             // Consume scroll to prevent default behavior of the graph editor
                             ui.ctx().input_mut(|i| {
                                 i.smooth_scroll_delta = egui::Vec2::ZERO;
                                 i.raw_scroll_delta = egui::Vec2::ZERO;
                             });
                        }
                    }
                }

                // Draw the grid
                self.draw_grid(ui);

                self.state.draw_graph_editor(
                    ui,
                    AllMyNodeTemplates,
                    &mut self.user_state,
                    Vec::default(),
                )
            })
            .inner;
        
        // Context Menu Logic
        let zoom = self.state.pan_zoom.zoom;
        let pan = self.state.pan_zoom.pan;
        
        if ctx.input(|i| i.pointer.secondary_clicked()) && !ctx.is_using_pointer() {
             self.context_menu_pos = ctx.pointer_interact_pos();
        }

        if let Some(pos) = self.context_menu_pos {
             let mut open = true;
             let window_response = egui::Window::new("NodeMenu")
                .fixed_pos(pos)
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.set_min_width(120.0);
                    ui.menu_button("New", |ui| {
                         if ui.button("MIDI Node").clicked() {
                            spawn_node(&mut self.state, &mut self.user_state, MyNodeTemplate::MidiNode, pos, pan, zoom);
                            ui.close_menu();
                            open = false;
                        }
                        if ui.button("Sound Source").clicked() {
                            spawn_node(&mut self.state, &mut self.user_state, MyNodeTemplate::SoundSource, pos, pan, zoom);
                            ui.close_menu();
                            open = false;
                        }
                        if ui.button("Effect").clicked() {
                            spawn_node(&mut self.state, &mut self.user_state, MyNodeTemplate::Effect, pos, pan, zoom);
                            ui.close_menu();
                            open = false;
                        }
                        if ui.button("Mixer").clicked() {
                            spawn_node(&mut self.state, &mut self.user_state, MyNodeTemplate::Mixer, pos, pan, zoom);
                            ui.close_menu();
                            open = false;
                        }
                    });
                    ui.separator();
                    if ui.button("Properties").clicked() {
                        ui.close_menu();
                         open = false;
                    }
                     if ui.button("Cancel").clicked() {
                         open = false;
                     }
                });
            
            if let Some(response) = window_response {
                 if response.response.clicked_elsewhere() {
                      open = false;
                 }
            }

            if !open {
                self.context_menu_pos = None;
            }
        }
    }
}

fn spawn_node(
    state: &mut MyEditorState,
    user_state: &mut MyGraphState,
    template: MyNodeTemplate,
    screen_pos: egui::Pos2,
    pan: egui::Vec2,
    zoom: f32,
) {
    let graph_pos = (screen_pos.to_vec2() - pan) / zoom; 
    let node_id = state.graph.add_node(
        template.node_graph_label(user_state),
        template.user_data(user_state),
        |graph, node_id| template.build_node(graph, user_state, node_id),
    );
    
    state.node_positions.insert(node_id, graph_pos.to_pos2());
}

pub struct AllMyNodeTemplates;

impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = MyNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            MyNodeTemplate::MidiNode,
            MyNodeTemplate::SoundSource,
            MyNodeTemplate::Effect,
            MyNodeTemplate::Mixer,
        ]
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Harmonia DAW",
        options,
        Box::new(|cc| Ok(Box::new(HarmoniaApp::new(cc)))),
    )
}
