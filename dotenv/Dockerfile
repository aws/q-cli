FROM node:buster

# Install dependencies
RUN apt update
RUN apt install -y python3 python3-pip && ln -sf python3 /usr/bin/python
RUN pip3 install --no-cache --upgrade pip setuptools
RUN apt install -y \
  make g++ libtool libtool-bin libc-dev perl \
  bash zsh \
  uuid-runtime netcat-openbsd jq

ENV HOME=/usr/home
RUN mkdir -p /Applications/Fig.app/
RUN mkdir -p /usr/home/app/ /usr/home/figterm/ /usr/home/.fig/

# Set up shell integrations.
COPY ./config/ /usr/home/config/
RUN cd /usr/home/config/ && bash ./tools/install_and_upgrade.sh local
COPY dotenv/bin/fig /usr/home/.fig/bin/

# Set up figterm.
COPY ./figterm/ /usr/home/figterm/
RUN cd /usr/home/figterm && make clean && make install

WORKDIR /usr/home/app/

COPY dotenv/package.json /usr/home/app/
RUN npm install

COPY ./dotenv .


CMD ["npm", "run", "test"]
