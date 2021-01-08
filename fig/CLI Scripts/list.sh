#!/usr/bin/env bash

ls ~/.fig/autocomplete | grep -v README | cut -d '.' -f 1
