

pub enum ElementState{
    Solid,
    Liquid,
    Gas,
    Plasma
}

/// An element is a substance that can exist in a solid, liquid, or gas state.
/// 
/// This should be configurable from config files.
pub struct Element{
    pub state: ElementState,
    pub id: String,
    pub name: String,
    pub specific_heat_capacity: f32,
}

impl Element{
    pub fn new(id: String, name: String, specific_heat_capacity: f32) -> Self {
        Self::default().with_id(id).with_name(name).with_specific_heat_capacity(specific_heat_capacity)
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_specific_heat_capacity(mut self, specific_heat_capacity: f32) -> Self {
        self.specific_heat_capacity = specific_heat_capacity;
        self
    }

    pub fn with_state(mut self, state: ElementState) -> Self {
        self.state = state;
        self
    }
}

impl Default for Element{
    fn default() -> Self {
        Self { id: "default".to_string(), name: "Default".to_string(), specific_heat_capacity: 0.0, state: ElementState::Solid }
    }
}