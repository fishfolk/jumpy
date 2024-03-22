use bitflags::bitflags;

bitflags! {
/// Flags for collision filtering, this is used for collision events.
pub struct CollisionGroup: u32 {
    // May not detect colliions
    const NONE = 0b0000;
    /// Default membership for bodies
    const DEFAULT= 0b0001;
    // All CollisionGroups are on body / or filtered for body
    const ALL = u32::MAX;
}
}

bitflags! {
/// Flags for filtering simulated collision for dynamics.
/// First [`CollisionGroup`] filters intersection, and then
/// if `SimulationGroup` flags do not intersect, collision event is generated,
/// but not contact forces.
pub struct SolverGroup: u32 {
    const NONE = 0b0000;
    // Solid world geometry like tiles go in this group
    const SOLID_WORLD = 0b0001;
    // Jump through world geometry. (Contacts will be modified so simulated objects only collide from above).
    const JUMP_THROUGH = 0b0010;
    // Dynamic bodies go in this group
    const DYNAMIC = 0b0100;
    const ALL = u32::MAX;
}
}
