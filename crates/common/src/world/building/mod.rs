
pub trait Building {

}

pub struct BuidlingDefinition {
    pub display_name: String,
    pub description: String,
}

pub struct DataDrivenBuildingDefinition {
    pub display_name: String,
    pub description: String,

    pub data: HashMap<String, String>,
}