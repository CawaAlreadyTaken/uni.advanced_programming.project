#!/bin/bash

# Controlla se è stato fornito un input
if [ -z "$1" ]; then
  echo "You must pass one argument."
  echo "Usage: ./full_git_push \"<String to insert in the commit>\""
  exit 1
fi

commit_string="$1"

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

# Funzione per fare commit nelle directory
git_commit_in_directories() {
  for dir in "${directories[@]}"; do
    cd "$dir" || exit 1

    git add --all
    git commit -m "$commit_string"

    git config pull.rebase false

    git pull --no-edit --commit origin main

    if [ $? -ne 0 ]; then
	echo "Conflicts while pulling $dir. Solve them and try again."
        exit 1
    fi

    cd - > /dev/null || exit 1

  done
}

# Funzione per fare push nelle directory
git_push_in_directories() {
  for dir in "${directories[@]}"; do
    cd "$dir" || exit 1
    git push
    cd - > /dev/null || exit 1
  done
}

# Commit of every working dir
git_commit_in_directories
# Commit of the parent dir
git add --all
git commit -m "$commit_string"
git pull --no-edit --commit origin main

if [ $? -ne 0 ]; then
    echo "Conflicts while pulling main repo. Solve them and try again."
    exit 1
fi

# Push of every working dir
git_push_in_directories
# Push of the root dir
git push

# General cargo update
./auto_cargo_update.sh

# Commit of every working dir
git_commit_in_directories
# Commit of the parent dir
git add --all
git commit -m "cargo updated"
git pull --no-edit --commit

if [ $? -ne 0 ]; then
    echo "Conflicts while pulling main repo. Solve them and try again."
    exit 1
fi

# Push of every working dir
git_push_in_directories
# Push of the root dir
git push

echo "Done! FrederickTheBest got your back! ~ comment not by FrederickTheBest"
