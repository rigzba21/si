use crate::CliResult;
use crate::SiCliError;
use docker_api::models::{ContainerSummary, ImageSummary};
use docker_api::opts::{ContainerListOpts, ImageListOpts, PullOpts};
use docker_api::Docker;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::cmp::min;
use std::string::ToString;

const REQUIRED_CONTAINER_LIST: &[&str] = &[
    "systeminit/jaeger",
    "systeminit/otelcol",
    "systeminit/postgres",
    "systeminit/nats",
    // "systeminit/sdf",
    // "systeminit/council",
    // "systeminit/veritech",
    // "systeminit/pinga",
    // "systeminit/web",
];

pub(crate) async fn downloaded_systeminit_containers_list() -> Result<Vec<ImageSummary>, SiCliError>
{
    let docker = Docker::unix("//var/run/docker.sock");
    let opts = ImageListOpts::builder().all(true).build();
    let mut containers = docker.images().list(&opts).await?;

    let containers: Vec<ImageSummary> = containers
        .drain(..)
        .filter(|c| {
            c.repo_tags
                .iter()
                .any(|t| t.starts_with("systeminit/") && t.ends_with(":stable"))
        })
        .collect();

    Ok(containers)
}

pub(crate) async fn running_systeminit_containers_list() -> Result<Vec<ContainerSummary>, SiCliError>
{
    let docker = Docker::unix("//var/run/docker.sock");
    let opts = ContainerListOpts::builder().all(true).build();
    let mut containers = docker.containers().list(&opts).await?;

    let containers: Vec<ContainerSummary> = containers
        .drain(..)
        .filter(|c| {
            c.image
                .as_ref()
                .is_some_and(|c| c.starts_with("systeminit/") && c.ends_with(":stable"))
                && c.state.as_ref().is_some_and(|c| c == "running")
        })
        .collect();

    Ok(containers)
}

pub(crate) async fn missing_containers() -> Result<Vec<String>, SiCliError> {
    let mut missing_containers = Vec::new();
    let containers = downloaded_systeminit_containers_list().await?;

    for required_container in REQUIRED_CONTAINER_LIST.iter().copied() {
        if !containers
            .iter()
            .any(|c| c.repo_tags.iter().all(|t| t.contains(required_container)))
        {
            missing_containers.push(required_container.to_string());
        }
    }

    Ok(missing_containers)
}

pub(crate) async fn get_non_running_containers() -> Result<Vec<String>, SiCliError> {
    let mut non_running_containers = Vec::new();
    let running_containers = running_systeminit_containers_list().await?;

    for required_container in REQUIRED_CONTAINER_LIST.iter().copied() {
        if !running_containers.iter().any(|c| {
            c.image
                .as_ref()
                .is_some_and(|c| c.contains(required_container))
        }) {
            non_running_containers.push(required_container.to_string());
        }
    }

    Ok(non_running_containers)
}

pub(crate) async fn download_missing_containers(missing_containers: Vec<String>) -> CliResult<()> {
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "{spinner:.red} [{elapsed_precise}] [{wide_bar:.yellow/blue}]",
    )
    .unwrap()
    .progress_chars("#>-");

    let total_size = 100123123;

    println!("Found {0} missing containers", missing_containers.len());

    let mut spawned = Vec::new();
    for missing_container in missing_containers {
        let pb = m.add(ProgressBar::new(total_size));
        pb.set_style(sty.clone());

        let mut message = "Downloading ".to_owned();
        message.push_str(missing_container.as_str());

        let h1 = tokio::spawn(async move {
            let docker = Docker::unix("//var/run/docker.sock");
            let mut downloaded = 0;

            let pull_opts = PullOpts::builder()
                .image(missing_container)
                .tag("stable")
                .build();
            let images = docker.images();
            let mut stream = images.pull(&pull_opts);
            while let Some(pull_result) = stream.next().await {
                match pull_result {
                    Ok(docker_api::models::ImageBuildChunk::PullStatus {
                        progress_detail, ..
                    }) => {
                        if let Some(progress_detail) = progress_detail {
                            let new = min(
                                downloaded + progress_detail.current.unwrap_or(0),
                                total_size,
                            );
                            downloaded = progress_detail.current.unwrap_or(0);
                            pb.set_position(new);
                        }
                    }
                    Ok(_) => {}
                    Err(e) => eprintln!("{e}"),
                }
            }
        });

        m.println(message).unwrap();

        spawned.push(h1);
    }

    for spawn in spawned {
        spawn.await.unwrap();
    }

    m.println("All containers successfully downloaded").unwrap();
    m.clear().unwrap();

    Ok(())
}
