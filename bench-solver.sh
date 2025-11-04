#!/usr/bin/env bash

set -euo pipefail

cmds=(
    "raphael-cli solve --recipe-id 35829 --stats 4900 4800 620 --level 100 --manipulation"
    "raphael-cli solve --recipe-id 35830 --stats 5428 5333 737 --level 100 --manipulation --heart-and-soul"
)

cargo install --path raphael-cli

echo "| Command | Time | Memory |"
echo "| ------- | ---: | -----: |"
for cmd in "${cmds[@]}"
do
    /usr/bin/time --format "| \`$cmd\` | %e s | %M kB |" bash -c "$cmd &> /dev/null" 2>&1
done
