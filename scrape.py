#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.requests python3Packages.tabulate

import requests
import tabulate

region = "us-west-2"

data = requests.get("https://raw.githubusercontent.com/vantage-sh/ec2instances.info/master/www/instances.json").json()
data = [
  (
    x["instance_type"],
    x["vCPU"],
    x["clock_speed_ghz"],
    x["memory"],
    float(x["pricing"][region]["linux"]["ondemand"])
  )
  for x in data
  # Some instances aren't available in the region that I usually use.
  if region in x["pricing"]
]
data = sorted(data, key=lambda y: f"{y[0].split('.')[0]} {y[4]}")
data = [
  (it, f"{vcpu} vCPU", cs, f"{mem}Gb", f"${price}/hr")
  for (it, vcpu, cs, mem, price) in data
]
open("instances.txt", "w").write(tabulate.tabulate(data, tablefmt="plain"))
