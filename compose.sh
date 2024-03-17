#!/bin/bash

set -e

docker compose up -d 
docker compose exec test-container /bin/bash

