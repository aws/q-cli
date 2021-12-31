FROM node

# Install dependencies
RUN echo "deb http://deb.debian.org/debian bullseye-backports main" >> /etc/apt/sources.list
RUN apt update
RUN apt install -y python3 python3-pip && ln -sf python3 /usr/bin/python
RUN pip3 install --no-cache --upgrade pip setuptools
RUN apt install -y \
  sudo make g++ libtool libtool-bin libc-dev perl \
  bash zsh fish \
  uuid-runtime netcat-openbsd jq \
  protobuf-compiler \
  vim valgrind \
  llvm clang
RUN apt install -y -t bullseye-backports golang-1.17-go
RUN npm install --global ts-proto

ENV HOME=/usr/home

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable --profile minimal -y

ENV FIG_BUNDLE_EXECUTABLES=/usr/home/.fig_bin
ENV PATH="${HOME}/.cargo/bin:/usr/lib/go-1.17/bin/:${PATH}"
ENV TMPDIR=/tmp/

WORKDIR /usr/home/app/

RUN mkdir -p /Applications/Fig.app/
RUN mkdir -p /usr/home/app/src/ \
  /usr/home/figterm/ \
  /usr/home/.fig/ \
  /usr/home/bin/ \
  /usr/home/figcli/bin/ \
  ${FIG_BUNDLE_EXECUTABLES}

# Set up test user
RUN adduser --disabled-password --home /usr/home test 
RUN adduser test sudo
# Prevent password prompts for sudo
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers
RUN chown -R test:test /usr/home/
USER test

# Set up testing infrastructure
COPY --chown=test:test ./dotenv/package.json .
RUN npm install

COPY --chown=test:test ./dotenv .

COPY --chown=test:test ./proto/ /usr/home/proto/
RUN protoc --plugin="$(npm bin -g)/protoc-gen-ts_proto" \
	   --ts_proto_opt=esModuleInterop=true \
	   --ts_proto_opt=oneof=unions \
	   --ts_proto_out="./src" \
		 --experimental_allow_proto3_optional \
	   -I/usr/home/proto "/usr/home/proto/local.proto"

RUN cp ./bin/fig $FIG_BUNDLE_EXECUTABLES/fig
RUN cp ./bin/fig $FIG_BUNDLE_EXECUTABLES/figcli

COPY --chown=test:test ./figcli2/ /usr/home/figcli/
RUN ls -lah /usr/home/figcli
RUN cd /usr/home/figcli && make clean && BUILT_PRODUCTS_DIR=$(pwd) make all

# Set up figterm.
COPY --chown=test:test ./figterm/ /usr/home/figterm/
RUN cd /usr/home/figterm && make clean && BUILT_PRODUCTS_DIR=${FIG_BUNDLE_EXECUTABLES} make all
RUN cp ${FIG_BUNDLE_EXECUTABLES}/figcli /usr/home/bin/figcli
RUN cp ${FIG_BUNDLE_EXECUTABLES}/figcli /usr/home/bin/fig

# Set up config repo.
COPY --chown=test:test ./config/ /usr/home/config/
RUN mkdir -p /usr/home/config/bin/ && cp -r /usr/home/.fig_bin/* /usr/home/config/bin/

# Provide easy script for child images to call to add fig to dotfiles.
RUN echo '!#/bin/bash\ncd /usr/home/config/ && ./tools/install_and_upgrade.sh local\n' > ~/install-fig
RUN chmod +x ~/install-fig

# Indicate src/ will be mounted to preserve things like local.ts that are generated in the container.
VOLUME /usr/home/app/src/

CMD ["bash"]
