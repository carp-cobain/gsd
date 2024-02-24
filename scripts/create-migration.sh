#!/bin/bash

if [[ $# -eq 0 ]] ; then
	echo 'A migration name is required'
	exit 1
fi

touch "migrations/$(date -u +"%Y%m%d%H%M%S")_$1.sql"
