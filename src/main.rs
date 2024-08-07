use polars::prelude::*;
use reqwest::header;
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;

type ProjectName = &'static str;
type ProjectId = u32;

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

fn get_project_info(
    json_result: &serde_json::Map<String, serde_json::Value>,
    project_names: &HashSet<ProjectName>,
) -> Result<(ProjectName, ProjectId), Box<dyn Error>> {
    let serde_json::Value::String(title) = json_result
        .get("title")
        .expect("There should be a title key")
    else {
        return Err("Expected project title in projects_info json to be a string".into());
    };
    if let Some(const_title_ref) = project_names.get(title as &str) {
        let serde_json::Value::Number(id) =
            json_result.get("id").expect("There should be an id key")
        else {
            return Err("Expected project id in projects_info json to be a number".into());
        };
        Ok((
            const_title_ref,
            id.as_u64().expect("The id should be a small integer") as u32,
        ))
    } else {
        return Err("Couldn't find project info in result object (each item in results array should contain project info)".into());
    }
}

#[derive(serde::Deserialize)]
struct ProjectsInfo {
    // count: i32,
    results: serde_json::Value,
}

async fn get_projects_hashmap(
    api_token: &str,
    client: &reqwest::Client,
    project_names: HashSet<ProjectName>,
) -> Result<HashMap<ProjectName, ProjectId>, Box<dyn Error>> {
    let query = format!("{}/api/projects/", HOST_NAME);
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
        if let Ok(project_info) = get_project_info(object, &project_names) {
            name_to_id.insert(project_info.0, project_info.1);
        }
    }
    Ok(name_to_id)
}

// NOTE:
// YOU MAY HAVE TO CHANGE AnnotationData and PredictionData 
// IT IS FOR TIME SERIES LABLES WITH integer START AND END (label studio does weird stuff so predictions have strings containing the integers)
#[derive(serde::Deserialize, Debug, Clone)]
struct PredictionData {
    start: String, 
    end: String,
    instant: bool,
    timeserieslabels: Vec<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct AnnotationData {
    start: u32, 
    end: u32,
    instant: bool,
    timeserieslabels: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
struct Prediction {
    id: String,
    from_name: String,
    to_name: String,
    value: PredictionData,
}

#[derive(serde::Deserialize, Debug)]
struct PredictionsSet {
    id: u32,
    created_ago: String,
    result: Vec<Prediction>,
}

type Predictions = Vec<PredictionsSet>;

#[derive(serde::Deserialize, Debug, Clone)]
struct Annotation {
    id: String,
    from_name: String,
    to_name: String,
    origin: String,
    value: AnnotationData,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct AnnotationsSet {
    id: u32,
    created_at: String,
    result: Vec<Annotation>,
    task: u32,
    project: u32
}

type Annotations = Vec<AnnotationsSet>;

#[derive(serde::Deserialize, Debug)]
struct TaskDataForParsing {
    id: u32,
    file_upload: String,
    annotations: Annotations,
    predictions: Vec<u32>,
    data: serde_json::Value,
}

#[derive(Debug)]
struct TaskData {
    id: u32,
    file_upload: String,
    annotations: Annotations,
    predictions: Predictions,
    data: serde_json::Value,
}

#[derive(Debug)]
struct ProjectData {
    id: u32,
    tasks: Vec<TaskData>,
}

async fn get_task_predictions(
    api_token: &str,
    client: &reqwest::Client,
    project_id: &ProjectId,
    task_id: &u32
) -> Result<Predictions, Box<dyn Error>> {
    let query = format!(
        "{}/api/predictions?project={}&task={}",
        HOST_NAME,
        project_id,
        task_id
    );
    let auth_header = format!("Token {}", api_token);
    let predictions: Predictions = client
        .get(query)
        .header(header::AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Predictions>()
        .await?;
    Ok(predictions)
}

async fn get_project_data(
    api_token: &str,
    client: &reqwest::Client,
    project_id: &ProjectId,
) -> Result<ProjectData, Box<dyn Error>> {
    let query = format!("{}/api/projects/{}/export?exportType=JSON&download_all_tasks=true", HOST_NAME, project_id);
    let auth_header = format!("Token {}", api_token);
    let project_data_parsed: Vec<TaskDataForParsing> = client
        .get(query)
        .header(header::AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Vec<TaskDataForParsing>>()
        .await?;
    let mut project_tasks: Vec<TaskData> = Vec::new();
    for task in project_data_parsed.iter() {
        let task_id = task.id;
        let predictions = get_task_predictions(api_token, client, project_id, &task_id).await?;
        let mut annotations = task.annotations.clone();
        for mut annotations_set in &mut annotations {
            annotations_set.result = annotations_set.result.iter().filter_map(|a| match a.origin == "manual" {
                false => None,
                true => Some(a.clone()),
            }).collect();
        }
        println!("{:?}", annotations);
        let task_data = TaskData {
            id: task_id,
            file_upload: task.file_upload.clone(),
            annotations: annotations,
            predictions: predictions,
            data: task.data.clone(),
        };
        project_tasks.push(task_data);
    }
    let project_data = ProjectData {
        id: *project_id,
        tasks: project_tasks,
    };
    Ok(project_data)
}

async fn get_all_project_data(
    api_token: &str,
    client: &reqwest::Client,
    project_map: HashMap<ProjectName, ProjectId>,
) -> Result<HashMap<ProjectName, ProjectData>, Box<dyn Error>> {
    let mut out_project_map = HashMap::new();
    for (name, id) in project_map.iter() {
        let project_data = get_project_data(api_token, client, id).await?;
        out_project_map.insert(*name, project_data);
    }
    Ok(out_project_map)
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
    let projects_map = get_projects_hashmap(api_token, &client, project_names_hashset).await?;
    let projects_map = get_all_project_data(api_token, &client, projects_map).await?;
    Ok(())
}
