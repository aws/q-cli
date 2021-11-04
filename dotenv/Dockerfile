FROM --platform=linux/arm64v8 arm64v8/node

# Install dependencies
RUN apt update
RUN apt install -y python3 python3-pip && ln -sf python3 /usr/bin/python
RUN pip3 install --no-cache --upgrade pip setuptools
RUN apt install -y \
  make g++ libtool libtool-bin libc-dev perl \
  bash zsh \
  uuid-runtime netcat-openbsd jq

WORKDIR /usr/home/app/

# Set up testing infrastructure
COPY ./dotenv/package.json .
RUN node -v
RUN npm install

# Copy mock fig executable over.
COPY ./dotenv .

ENV HOME=/usr/home
ENV FIG_BUNDLE_EXECUTABLES=/usr/home/.fig_bin
RUN mkdir -p /Applications/Fig.app/
RUN mkdir -p /usr/home/app/ /usr/home/figterm/ /usr/home/.fig/ ${FIG_BUNDLE_EXECUTABLES}

RUN cp ./bin/fig ${FIG_BUNDLE_EXECUTABLES}/figcli
RUN cp ./bin/fig ${FIG_BUNDLE_EXECUTABLES}/fig

# Set up figterm.
COPY ./figterm/ /usr/home/figterm/
RUN cd /usr/home/figterm && make clean && BUILT_PRODUCTS_DIR=${FIG_BUNDLE_EXECUTABLES} make all

# Set up config repo.
COPY ./config/ /usr/home/config/
RUN mkdir -p /usr/home/config/bin/ && cp -r /usr/home/.fig_bin/* /usr/home/config/bin/
# Provide easy script for child images to call to add fig to dotfiles.
RUN echo '!#/bin/bash\ncd /usr/home/config/ && ./tools/install_and_upgrade.sh local\n' > ~/install-fig
RUN chmod +x ~/install-fig

CMD ["bash"]
