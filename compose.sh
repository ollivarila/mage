#!/bin/bash

set -e

docker compose up -d 
sleep 1
docker compose exec test-container /bin/bash

