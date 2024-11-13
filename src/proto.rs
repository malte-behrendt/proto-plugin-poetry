use rustc_hash::FxHashMap;
use std::{collections::HashMap, path::PathBuf};

use extism_pdk::*;
use proto_pdk::*;
use regex::Regex;

static NAME: &str = "Poetry";

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
    fn from_virtual_path(path: String) -> String;
}

#[plugin_fn]
pub fn register_tool(_: ()) -> FnResult<Json<ToolMetadataOutput>> {
    Ok(Json(ToolMetadataOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        minimum_proto_version: Some(Version::new(0, 42, 0)),
        ..ToolMetadataOutput::default()
    }))
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    Ok(Json(DownloadPrebuiltOutput {
        download_url: format!("https://github.com/python-poetry/poetry/releases/download/{0}/poetry-{0}-py3-none-any.whl", input.context.version.to_string()),
        ..DownloadPrebuiltOutput::default()
    }))
}

#[plugin_fn]
pub fn unpack_archive(Json(input): Json<UnpackArchiveInput>) -> FnResult<()> {
    let mut result = exec_command!("which", ["python"]);

    if result.exit_code != 0 {
        return Err(plugin_err!(
            "Python not found. Please install Python before installing Poetry."
        ));
    }

    let output_path_host_str = real_path!(buf, PathBuf::from(input.output_dir.as_path()))
        .display()
        .to_string();
    let mut env: FxHashMap<String, String> = FxHashMap::default();
    env.insert("POETRY_HOME".into(), output_path_host_str.clone());

    result = exec_command!(
        input,
        ExecCommandInput {
            command: "python".into(),
            args: vec!["-m".into(), "venv".into(), output_path_host_str.clone()],
            ..ExecCommandInput::default()
        }
    );

    if result.exit_code != 0 {
        return Err(plugin_err!("Failed to set up Python venv"));
    }

    let input_file_path_host_str = real_path!(buf, PathBuf::from(input.input_file.as_path()))
        .display()
        .to_string();

    result = exec_command!(
        input,
        ExecCommandInput {
            command: format!("{0}/bin/pip", output_path_host_str),
            args: vec!["install".into(), input_file_path_host_str,],
            env: env.clone(),
            ..ExecCommandInput::default()
        }
    );

    if result.exit_code != 0 {
        return Err(plugin_err!(format!(
            "Failed to install Poetry. Please investigate: {0}\n{1}",
            result.stdout, result.stderr
        )));
    }

    // For ensuring to use the correct Python version
    result = exec_command!(
        input,
        ExecCommandInput {
            command: format!("{0}/bin/poetry", output_path_host_str),
            args: vec![
                "config".into(),
                "virtualenvs.prefer-active-python".into(),
                "true".into()
            ],
            env,
            ..ExecCommandInput::default()
        }
    );

    if result.exit_code != 0 {
        return Err(plugin_err!(format!(
            "Failed to configure Poetry. Please investigate: {0}",
            result.stderr
        )));
    }

    Ok(())
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    Ok(Json(LocateExecutablesOutput {
        exes: HashMap::from_iter([(
            "poetry".into(),
            ExecutableConfig {
                exe_path: Some(PathBuf::from("bin/poetry")),
                no_shim: false,
                primary: true,
                ..ExecutableConfig::default()
            },
        )]),
        // globals_lookup_dirs: vec!["$POETRY_HOME/bin".into()],
        ..LocateExecutablesOutput::default()
    }))
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let version_regex: Regex = Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+$")?;

    let tags = load_git_tags("https://github.com/python-poetry/poetry")?
        .iter()
        .filter(|tag| version_regex.is_match(tag))
        .map(|tag| tag.into())
        .collect::<Vec<_>>();

    Ok(Json(LoadVersionsOutput::from(tags)?))
}

// TODO: detect_version_files
// TODO: parse_version_file
