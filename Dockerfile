FROM rust:1.73 as builder

# INSTALL ALSA Dev dependencies
RUN apt-get update && apt-get install -y libasound2-dev

# INSTALL TAURI DEPENDENCIES
RUN apt-get install -y libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Install NodeJS and NPM
RUN apt-get install -y ca-certificates curl gnupg && \
	mkdir -p /etc/apt/keyrings && \
	curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg

RUN NODE_MAJOR=20 && \
	echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_MAJOR.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list
RUN apt-get update && \
	apt-get install nodejs -y
RUN npm install -g @tauri-apps/cli

# Install x11 utils -> need to test if necessary
RUN apt-get install -y -qqy x11-apps

WORKDIR /usr/src/myapp/passeri-gui

ENTRYPOINT npm install && npm run tauri dev

# run  `xhost +` before

# docker run --rm \
# 			-v "$PWD/tauri_rythmo_sync":/usr/src/myapp \	# project folder
# 			-v $HOME/.Xauthority:/root/.Xauthority:rw \		# x11 forwarding
# 			-v /tmp/.X11-unix:/tmp/.X11-unix \				# x11 forwarding
# 			-v /dev/snd/seq:/dev/snd/seq \					# midi ports forwarding
# 			--net=host \									# use the same ip as host
# 			-e DISPLAY=$DISPLAY \							# share the display ID
# 			57cc1e2292fc									# the name of the Docker Image

# run `xhost -` after
