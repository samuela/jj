use rusoto_ec2::Ec2;
use skim::prelude::Skim;
use skim::prelude::SkimItemReader;
use skim::prelude::SkimOptionsBuilder;
use std::io::Cursor;
use std::process::exit;
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, PartialEq)]
enum InstanceState {
  Pending,
  Running,
  ShuttingDown,
  Terminated,
  Stopping,
  Stopped,
}

impl FromStr for InstanceState {
  type Err = ();

  fn from_str(input: &str) -> Result<InstanceState, Self::Err> {
    // See https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_InstanceState.html
    match input {
      "pending" => Ok(InstanceState::Pending),
      "running" => Ok(InstanceState::Running),
      "shutting-down" => Ok(InstanceState::ShuttingDown),
      "terminated" => Ok(InstanceState::Terminated),
      "stopping" => Ok(InstanceState::Stopping),
      "stopped" => Ok(InstanceState::Stopped),
      _ => Err(()),
    }
  }
}

async fn get_instance_state(ec2client: &rusoto_ec2::Ec2Client, instance_id: &str) -> InstanceState {
  InstanceState::from_str(
    ec2client
      .describe_instance_status(rusoto_ec2::DescribeInstanceStatusRequest {
        include_all_instances: Some(true),
        instance_ids: Some(vec![instance_id.to_string()]),
        ..Default::default()
      })
      .await
      .expect("borked calling describe_instance_status")
      .instance_statuses
      .expect("borked getting instance_statuses")[0]
      .instance_state
      .as_ref()
      .expect("borked getting instance_state")
      .name
      .as_ref()
      .expect("borked getting name"),
  )
  .expect("got illegal instance state value")
}

fn ssh(hostname: &str) -> std::io::Result<()> {
  eprintln!("🚀 waiting for an SSH connection...");
  // See https://stackoverflow.com/questions/53477846/start-another-program-then-quit
  // Note: hardcoding doodoo hostname for now...
  let exit_status = Command::new("ssh")
    .args(&[hostname.to_string()])
    .spawn()?
    .wait()
    .expect("ssh borked itself");

  if exit_status.success() {
    Ok(())
  } else {
    Err(std::io::Error::new(
      std::io::ErrorKind::Other,
      "ssh exited with non-zero exit code",
    ))
  }
}

async fn select_and_start_instance(
  aws: &rusoto_ec2::Ec2Client,
  instance_id: &str,
  hostname: &str,
) -> std::io::Result<()> {
  let options = SkimOptionsBuilder::default()
    // We already sort in the script that builds "instances.txt".
    .nosort(true)
    .exact(true)
    .build()
    .expect("borked building SkimOptions");

  let instance_types = include_str!("../instances.txt");

  let item_reader = SkimItemReader::default();
  let items = item_reader.of_bufread(Cursor::new(instance_types));
  let skim_output = Skim::run_with(&options, Some(items)).expect("borked getting skim_output");
  if skim_output.is_abort {
    eprintln!("Aborting...");
    exit(1);
  }

  let selected_items = skim_output.selected_items;

  // The skim library doesn't offer the cleanest interface here. It always
  // returns a Vec even when multi-selection is off.
  assert_eq!(selected_items.len(), 1);
  let selected_type = selected_items[0]
    .text()
    .split_once(' ')
    .expect("borked splitting selected line")
    .0
    .to_string();

  eprintln!("🍩 resizing instance...");
  // TODO: check that instance is in stopped state first, can't resize otherwise.
  aws
    .modify_instance_attribute(rusoto_ec2::ModifyInstanceAttributeRequest {
      instance_id: instance_id.to_string(),
      instance_type: Some(rusoto_ec2::AttributeValue {
        value: Some(selected_type),
      }),
      ..rusoto_ec2::ModifyInstanceAttributeRequest::default()
    })
    .await
    .expect("could not resize instance");

  eprintln!("🏃 starting instance...");
  aws
    .start_instances(rusoto_ec2::StartInstancesRequest {
      instance_ids: vec![instance_id.to_string()],
      ..rusoto_ec2::StartInstancesRequest::default()
    })
    .await
    .expect("could not start instance");

  ssh(hostname)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // TODO:
  //  * add `jj status` command. add --watch option?
  //  * add `jj stop` command?
  let instance_id = std::env::var("JJ_INSTANCE_ID").expect("env var JJ_INSTANCE_ID not set");
  let hostname = std::env::var("JJ_HOSTNAME").expect("env var JJ_HOSTNAME not set");

  // Note: Hardcoding us-west-1 for now...
  let aws = rusoto_ec2::Ec2Client::new(rusoto_signature::region::Region::UsWest1);

  let instance_state = get_instance_state(&aws, &instance_id).await;
  match instance_state {
    InstanceState::Pending | InstanceState::Running => ssh(&hostname),
    InstanceState::Stopped => select_and_start_instance(&aws, &instance_id, &hostname).await,
    InstanceState::Stopping | InstanceState::ShuttingDown => {
      eprintln!("🛑 waiting for instance to finish shutting down...");
      loop {
        sleep(Duration::from_secs(5)).await;
        if get_instance_state(&aws, &instance_id).await == InstanceState::Stopped {
          break;
        }
      }
      select_and_start_instance(&aws, &instance_id, &hostname).await
    }
    _ => {
      eprintln!(
        "don't know what do with the current instance state: {:?}",
        instance_state
      );
      exit(1);
    }
  }
}
