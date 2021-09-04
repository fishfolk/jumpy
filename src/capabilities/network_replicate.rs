use macroquad::experimental::scene::HandleUntyped;

#[derive(Clone, Copy)]
pub struct NetworkReplicate {
    /// Analogue of "fixed_update", but is called from network system
    /// when all the inputs are aligned and it is time to one frame advance
    /// the simulation
    pub network_update: fn(node: HandleUntyped),
}
