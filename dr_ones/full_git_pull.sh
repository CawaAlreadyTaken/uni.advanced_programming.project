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
    git pull

    cd - > /dev/null || exit 1

  done
}


git_pull_in_directories

# To pull docs
cd ..
git config pull.rebase false
git pull
