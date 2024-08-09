// use polars::prelude::*;
use std::env;


// NOTE: EDIT THESE
const PROJECT_NAMES: [ProjectName; 9] = [
    "Operational Mode",
    "Geometry State",
    "Pressure",
    "Torque",
    "Temperature",
    "Second Temperature",
    "Gamma",
    "Implausible Incline",
    "Implausible Azimuth",
];

// NOTE: EDIT THIS
const HOST_NAME: &'static str = "https://label.apex.manifold.group";



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let api_token: &str = args
        .get(1)
        .expect("Please pass the api token as the first command line argument");
    let project_names_hashset: HashSet<ProjectName> = HashSet::from(PROJECT_NAMES);
    // for project_name in PROJECT_NAMES { project_names_hashset.insert(project_name); }
    let client = reqwest::Client::new();
    let projects_map = get_projects_hashmap(api_token, &client, project_names_hashset).await?;
    let projects_map = get_all_project_data(api_token, &client, projects_map).await?;
    Ok(())
}
