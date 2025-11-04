#!/usr/bin/env bash

set -euxo pipefail

cmds=(
    "solve -r 35829 -s 4900 4800 620 --manipulation" # Rarefied Tacos de Carne Asada
    "solve -r 35829 -s 4900 4800 620 --manipulation --adversarial" # Rarefied Tacos de Carne Asada
    "solve -r 35830 -s 5428 5333 737 --manipulation --heart-and-soul" # Archeo Kingdom Broadsword
)

cargo install --path raphael-cli

echo "| Command | Time | Memory |"
echo "| ------- | ---: | -----: |"
for cmd in "${cmds[@]}"
do
    /usr/bin/time --format "| \`$cmd\` | %e s | %M kB |" bash -c "raphael-cli $cmd &> /dev/null" 2>&1
done
