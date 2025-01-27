#!/bin/bash

# Elenco delle directory da processare
directories=(
  "client"
  "server"
  "host_node"
  "simulation_controller"
  "network_node"
  "drone"
  "network_initializer"
)

# Function to pull from every dir
git_pull_in_directories() {
  for dir in "${directories[@]}"; do
    cd "$dir" || exit 1

    git config pull.rebase false
    git pull --no-edit --commit origin main

    if [ $? -ne 0 ]; then
	echo "Conflicts while pulling $dir. Solve them and try again."
        exit 1
    fi

    cd - > /dev/null || exit 1

  done
}


git_pull_in_directories

./auto_cargo_update.sh

# To pull docs
cd ..
git config pull.rebase false
git pull --no-edit --commit origin main


if [ $? -ne 0 ]; then
    echo "Conflicts while pulling main repo. Solve them and try again."
    exit 1
fi
