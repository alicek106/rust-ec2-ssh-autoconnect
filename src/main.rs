use aws_sdk_ec2::{Error, Region, Client};
use std::fs::File;
use clap::Command;

fn cli() -> Command<'static> {
    Command::new("ec2-connect")
        .about("A fictional versioning CLI")
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("list")
                .about("List all ec2 instances")
        )
}

async fn list_ec2_instances(client: &Client) -> Result<(), Error> {
    println!("{:w$}{:w2$}{:w3$}{}", "Instance ID", "Instance Name", "IP Address", "Status", w = 25, w2 = 20, w3 = 15);
    let resp = client
        .describe_instances()
        .send()
        .await?;

    for reservation in resp.reservations().unwrap_or_default() {
        for instance in reservation.instances().unwrap_or_default() {
            let instance_id = instance.instance_id().unwrap();
            let instance_state = instance.state().unwrap().name().unwrap();
            let public_ip = match instance.public_ip_address() {
                Some(s) => s,
                None => "Unknown",
            };

            let tags = instance.tags().unwrap();
            let mut instance_name = "";
            for tag in tags {
                if "Name" == tag.key.as_ref().unwrap() {
                    instance_name = (&tag.value.as_ref().unwrap()).clone();
                }
            }
            println!("{:w$}{:w2$}{:w3$}{:?}", instance_id, instance_name, public_ip, instance_state, w = 25, w2 = 20, w3 = 15);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 이 경로는 고정
    let config_file_path = "/etc/ec2_connect_config.yaml";

    let f = File::open(config_file_path).unwrap();
    let data: serde_yaml::Value = serde_yaml::from_reader(f).unwrap();

    let region: &'static str = Box::leak(data["spec"]["region"].as_str().unwrap().to_string().into_boxed_str());
    let region_provider = Region::new(region);

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(_) => {
            list_ec2_instances(&client).await
        }
        _ => unreachable!()
    }
}