use reqwest::header;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::env;

pub struct Config {
    api_token: String,
    host_name: String,
    project_names: HashSet<String>,
}

type ProjectName = String;
type ProjectId = u32;

const PROJECT_NAMES_ENV_VAR_NAME: &'static str = "LSRS_PROJECT_NAMES";
const HOST_NAME_ENV_VAR_NAME: &'static str = "LSRS_HOST_NAME";
const API_TOKEN_ENV_VAR_NAME: &'static str = "LSRS_API_TOKEN";

impl Config {
    pub fn build(args: &mut Vec<String>) -> Result<Config, Box<dyn Error>> {
        let Some(project_names_string) = Config::get_arg(args, PROJECT_NAMES_ENV_VAR_NAME) else {return Err(format!("Please pass project_names list as string (see README) as either last command line arg or as an environment variable {}", PROJECT_NAMES_ENV_VAR_NAME).into())};
        let Some(host_name) = Config::get_arg(args, HOST_NAME_ENV_VAR_NAME) else {return Err(format!("Please pass host_name as string (see README) as either last command line arg or as an environment variable {}", HOST_NAME_ENV_VAR_NAME).into())};
        let Some(api_token) = Config::get_arg(args, API_TOKEN_ENV_VAR_NAME) else {return Err(format!("Please pass api_token as string (see README) as either last command line arg or as an environment variable {}", API_TOKEN_ENV_VAR_NAME).into())};
        let project_names = Config::parse_project_names(project_names_string);
        Ok(Config {
            api_token,
            host_name,
            project_names
        })
    }

    fn parse_project_names(project_names_string: String) -> HashSet<ProjectName> {
        HashSet::<ProjectName>::from_iter(project_names_string.split(",").map(|s| s.to_string()))
    }

    // Returns env var if it is set else pops the last value from args
    fn get_arg(args: &mut Vec<String>, env_var_name: &str) -> Option<String> {
        let arg = match env::var(env_var_name) {
            Ok(s) => s,
            Err(_) => args.pop()?,
        };
        Some(arg)
    }
}


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
            (*const_title_ref).clone(),
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
    host_name: &str
) -> Result<HashMap<ProjectName, ProjectId>, Box<dyn Error>> {
    let query = format!("{}/api/projects/", host_name);
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
    task_id: &u32,
    host_name: &str
) -> Result<Predictions, Box<dyn Error>> {
    let query = format!(
        "{}/api/predictions?project={}&task={}",
        host_name,
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
    host_name: &str
) -> Result<ProjectData, Box<dyn Error>> {
    let query = format!("{}/api/projects/{}/export?exportType=JSON&download_all_tasks=true", host_name, project_id);
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
        let predictions = get_task_predictions(api_token, client, project_id, &task_id, host_name).await?;
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
    host_name: &str
) -> Result<HashMap<ProjectName, ProjectData>, Box<dyn Error>> {
    let mut out_project_map = HashMap::new();
    for (name, id) in project_map.iter() {
        let project_data = get_project_data(api_token, client, id, host_name).await?;
        out_project_map.insert((*name).to_string(), project_data);
    }
    Ok(out_project_map)
}

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let projects_map = get_projects_hashmap(&config.api_token, &client, config.project_names, &config.host_name).await?;
    let projects_map = get_all_project_data(&config.api_token, &client, projects_map, &config.host_name).await?;
    Ok(())
}