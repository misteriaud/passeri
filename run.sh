#!/bin/bash

docker build --tag 'tauri_dev' .
xhost +
docker run --rm -v .:/usr/src/myapp \
	--net=host \
	-e DISPLAY=$DISPLAY \
	-v $HOME/.Xauthority:/root/.Xauthority:rw \
	-v /tmp/.X11-unix:/tmp/.X11-unix \
	-v /dev/snd/seq:/dev/snd/seq \
	tauri_dev
