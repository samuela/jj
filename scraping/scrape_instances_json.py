#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.requests

"""Script to generate an instances.json file with all of the EC2 pricing info.

We download https://github.com/vantage-sh/ec2instances.info and run the build in
order to scrape everything into instances.json.

Run `render_instances_tables.py` after this.
"""

import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

import requests

scraping_dir = Path(__file__).parent.resolve()

# See https://stackoverflow.com/a/8378458.
# At the time of writing (2022-03-29), the latest commit is 5273920429f154d22534701383cc03118900386b.
resp = requests.get("https://api.github.com/repos/vantage-sh/ec2instances.info/tarball", stream=True)
with tempfile.TemporaryDirectory() as tmpdir:
  tarfile.open(fileobj=resp.raw, mode="r|gz").extractall(tmpdir)
  # We should only ever get one directory out of the tarball.
  assert len(list(Path(tmpdir).iterdir())) == 1
  repodir = next(Path(tmpdir).iterdir())

  print("Scraping... This takes a few minutes...")
  subprocess.check_call(["nix-shell", scraping_dir / "ec2instances-shell.nix", "--run", "invoke build"], cwd=repodir)

  (scraping_dir / "data").mkdir(exist_ok=True)
  shutil.copy(repodir / "www"/ "instances.json", scraping_dir / "data" / "instances.json")
