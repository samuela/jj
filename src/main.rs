use rusoto_ec2::Ec2;
use skim::prelude::Skim;
use skim::prelude::SkimItemReader;
use skim::prelude::SkimOptionsBuilder;
use std::{
  io::Cursor,
  process::{exit, Command},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  let instance_id = std::env::var("JJ_INSTANCE_ID").expect("env var JJ_INSTANCE_ID not set");

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

  // Note: Hardcoding us-west-1 for now...
  let aws = rusoto_ec2::Ec2Client::new(rusoto_signature::region::Region::UsWest1);

  println!("⚜️ resizing instance...");
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

  println!("🏃 starting instance...");
  aws
    .start_instances(rusoto_ec2::StartInstancesRequest {
      instance_ids: vec![instance_id.to_string()],
      ..rusoto_ec2::StartInstancesRequest::default()
    })
    .await
    .expect("could not start instance");

  println!("🚀 waiting for an SSH connection...");
  // See https://stackoverflow.com/questions/53477846/start-another-program-then-quit
  // Note: hardcoding doodoo hostname for now...
  Command::new("ssh")
    .args(&["doodoo"])
    .spawn()?
    .wait()
    .expect("ssh borked itself");

  Ok(())
}
