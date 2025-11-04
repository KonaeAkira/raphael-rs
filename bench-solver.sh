#!/usr/bin/env bash

set -euxo pipefail

cmds=(
    "solve -r 35829 -s 4900 4800 620 -l 100 --manipulation"
    # "solve -r 35830 -s 5428 5333 737 -l 100 --manipulation --heart-and-soul"
)

cargo install --path raphael-cli

echo "| Command | Time | Memory |"
echo "| ------- | ---: | -----: |"
for cmd in "${cmds[@]}"
do
    /usr/bin/time --format "| \`$cmd\` | %e s | %M kB |" bash -c "raphael-cli $cmd &> /dev/null" 2>&1
done
