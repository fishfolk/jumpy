use super::{EditorAction, EditorContext, EditorTool, EditorToolParams, Map};

#[derive(Default)]
pub struct PlacementTool {
    params: EditorToolParams,
}

impl PlacementTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Placement Tool".to_string(),
            ..Default::default()
        };

        PlacementTool { params }
    }
}

impl EditorTool for PlacementTool {
    fn get_params(&self) -> &EditorToolParams {
        &self.params
    }

    fn get_action(&mut self, _map: &Map, _ctx: &EditorContext) -> Option<EditorAction> {
        println!("TOOL");
        None
    }
}
