#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.requests python3Packages.tabulate

import json
from pathlib import Path

import requests
import tabulate

scraping_data_dir = Path(__file__).parent.resolve() / "data"

# See https://docs.rs/rusoto_signature/latest/src/rusoto_signature/region.rs.html#234-268.
regions = ["ap-east-1", "ap-northeast-1", "ap-northeast-2", "ap-northeast-3", "ap-south-1", "ap-southeast-1", "ap-southeast-2", "ca-central-1", "eu-central-1", "eu-west-1", "eu-west-2", "eu-west-3", "eu-north-1", "eu-south-1", "me-south-1", "sa-east-1", "us-east-1", "us-east-2", "us-west-1", "us-west-2", "us-gov-east-1", "us-gov-west-1", "cn-north-1", "cn-northwest-1", "af-south-1"]

all_data = json.load(open(scraping_data_dir / "instances.json"))
for region in regions:
  data = [
    (
      x["instance_type"],
      x["vCPU"],
      x["clock_speed_ghz"],
      x["memory"],
      float(x["pricing"][region]["linux"]["ondemand"])
    )
    for x in all_data
    # Not every instance is available in every region...
    if region in x["pricing"]
  ]
  data = sorted(data, key=lambda y: f"{y[0].split('.')[0]} {y[4]}")
  data = [
    (it, f"{vcpu} vCPU", cs, f"{mem}Gb", f"${price}/hr")
    for (it, vcpu, cs, mem, price) in data
  ]
  open(scraping_data_dir / f"{region}-instances-table.txt", "w").write(tabulate.tabulate(data, tablefmt="plain"))
