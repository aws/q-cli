#!/usr/bin/env bash

while IFS= read -ra line; do 
    IFS=\":. read -ra file <<<"${line[@]}"
    echo "${file[4]}"
done <<<"$(curl \
  -H "Accept: application/vnd.github.v3+json" \
  https://api.github.com/repos/withfig/autocomplete/contents/src | grep "name")"
