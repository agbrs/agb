#!/bin/bash

docker build . -t rustgba-dev
docker run --rm --volume $PWD:/build -it --workdir "/build" rustgba-dev