use aws_sdk_ec2::{Error, Region, Client};
use std::fs::File;
use clap::{arg, Command};

fn cli() -> Command<'static> {
    Command::new("ec2-connect")
        .bin_name("ec2-connect")
        .about("A fictional versioning CLI")
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("list")
                .about("List all ec2 instances")
        )
        .subcommand(
            Command::new("start")
                .about("Start a ec2 instance")
                .arg(arg!(<NAME> "Instance name to start"))
        )
        .subcommand(
            Command::new("stop")
                .about("Stop a ec2 instance")
                .arg(arg!(<NAME> "Instance name to stop"))
        )
        .subcommand(
            Command::new("connect")
                .about("Connect to a ec2 instance using SSH")
                .arg(arg!(<NAME> "Instance name to connect"))
        )
        .subcommand(
            Command::new("group")
                .args_conflicts_with_subcommands(true)
                .subcommand(Command::new("start").arg(arg!([GROUP_NAME])))
                .subcommand(Command::new("stop").arg(arg!([GROUP_NAME])))
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
            // instance_state -> string 어떻게 변환?
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

    // region을 가져와서 static lifetime으로 변환
    let region: &'static str = Box::leak(data["spec"]["region"].as_str().unwrap().to_string().into_boxed_str());
    let region_provider = Region::new(region);

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("list", _)) => {
            list_ec2_instances(&client).await
        },

        // TODO: 구현하기
        Some(("start", sub_matches)) => {
            println!("start -> {}", sub_matches.get_one::<String>("NAME").expect("required"));
            Ok(())
        },
        Some(("stop", sub_matches)) => {
            println!("stop -> {}", sub_matches.get_one::<String>("NAME").expect("required"));
            Ok(())
        },
        Some(("connect", sub_matches)) => {
            println!("connect -> {}", sub_matches.get_one::<String>("NAME").expect("required"));
            Ok(())
        },
        Some(("group", sub_matches)) => {
            let group_command = sub_matches.subcommand().unwrap();
            match group_command {
                ("start", sub_matches) => {
                    let group_name = sub_matches.get_one::<String>("GROUP_NAME");
                    println!("starting group for {:?}", group_name)
                }
                ("stop", sub_matches) => {
                    let group_name = sub_matches.get_one::<String>("GROUP_NAME");
                    println!("stopping group for {:?}", group_name)
                }
                (name, _) => {
                    unreachable!("Unsupported subcommand `{}`", name)
                }
            };
            Ok(())
        }
        _ => unreachable!()
    }
}