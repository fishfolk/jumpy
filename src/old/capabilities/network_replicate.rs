use macroquad::experimental::scene::HandleUntyped;

#[derive(Clone, Copy)]
pub struct NetworkReplicate {
    pub network_update: fn(node: HandleUntyped),
}
