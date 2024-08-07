use polars::prelude::*;
use reqwest::header;
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;

#[derive(Eq, Hash, PartialEq)]
struct ProjectName<'a>(&'a str);
struct ProjectId(u32);

const PROJECT_NAMES: [ProjectName; 9] = [
    ProjectName("Operational Mode"),
    ProjectName("Geometry State"),
    ProjectName("Pressure"),
    ProjectName("Torque"),
    ProjectName("Temperature"),
    ProjectName("Second Temperature"),
    ProjectName("Gamma"),
    ProjectName("Implausible Incline"),
    ProjectName("Implausible Azimuth"),
];

#[derive(serde::Deserialize)]
struct ProjectsInfo {
    // count: i32,
    results: serde_json::Value,
}


fn get_project_info<'a>(
    json_result: &serde_json::Map<String, serde_json::Value>,
    project_names: &'a HashSet<ProjectName>
) -> Result<(ProjectName<'a>, ProjectId), Box<dyn Error>> {
    let serde_json::Value::String(title) =
        json_result.get("title").expect("There should be a title key")
    else {
        return Err("Expected project title in projects_info json to be a string".into());
    };
    if let Some(const_title_ref) = project_names.get(&ProjectName(title as &str)) {
        let serde_json::Value::Number(id) = json_result.get("id").expect("There should be an id key")
        else {
            return Err("Expected project id in projects_info json to be a number".into());
        };

        Ok((
            *const_title_ref,
            ProjectId(id.as_u64().expect("The id should be a small integer") as u32),
        ))
    } else {
        return Err("Couldn't find project info in result object (each item in results array should contain project info)".into());
    }
}

async fn get_projects_hashmap<'a>(
    api_token: &str,
    client: &reqwest::Client,
    project_names: &'a HashSet<ProjectName>,
) -> Result<HashMap<ProjectName<'a>, ProjectId>, Box<dyn Error>> {
    let query = "https://label.apex.manifold.group/api/projects/";
    let auth_header = format!("Token {}", api_token);
    let projects_info = client
        .get(query)
        .header(header::AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<ProjectsInfo>()
        .await?;
    let serde_json::Value::Array(arr) = projects_info.results else {
        return Err("Expected projects_info json to have a results array at top level".into());
    };
    let mut name_to_id = HashMap::new();
    for item in &arr {
        let serde_json::Value::Object(object) = item else {
            return Err(
                "Expected each item results array in projects_info json to be an object".into(),
            );
        };
        let project_info = get_project_info(object, project_names)?;
        name_to_id.insert(project_info.0, project_info.1);
    }
    Ok(name_to_id)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let api_token: &str = args
        .get(1)
        .expect("Please pass the api token as the first command line argument");
    let project_names_hashset: HashSet<ProjectName> = HashSet::from(PROJECT_NAMES);
    // for project_name in PROJECT_NAMES { project_names_hashset.insert(project_name); }
    let client = reqwest::Client::new();
    get_projects_hashmap(api_token, &client, &project_names_hashset).await?;
    Ok(())
}
