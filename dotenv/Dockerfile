FROM node

# Install dependencies
RUN echo "deb http://deb.debian.org/debian bullseye-backports main" >> /etc/apt/sources.list
RUN apt update
RUN apt install -y python3 python3-pip && ln -sf python3 /usr/bin/python
RUN pip3 install --no-cache --upgrade pip setuptools
RUN apt install -y \
  make g++ libtool libtool-bin libc-dev perl \
  bash zsh \
  uuid-runtime netcat-openbsd jq \
  protobuf-compiler \
  vim valgrind
RUN apt install -y -t bullseye-backports golang-1.17-go

WORKDIR /usr/home/app/
ENV HOME=/usr/home
ENV FIG_BUNDLE_EXECUTABLES=/usr/home/.fig_bin
ENV PATH=/usr/lib/go-1.17/bin/:$PATH
ENV TMPDIR=/tmp/

# Set up testing infrastructure
COPY ./dotenv/package.json .
RUN node -v
RUN npm install

# Copy mock fig executable over.
COPY ./dotenv .

RUN mkdir -p /Applications/Fig.app/
RUN mkdir -p /usr/home/app/ \
  /usr/home/figterm/ \
  /usr/home/.fig/ \
  /usr/home/bin/ \
  /usr/home/figcli/bin/ \
  ${FIG_BUNDLE_EXECUTABLES}

RUN cp ./bin/fig $FIG_BUNDLE_EXECUTABLES/fig
RUN cp ./bin/fig $FIG_BUNDLE_EXECUTABLES/figcli

COPY ./proto/ /usr/home/proto/
RUN protoc --plugin="./node_modules/.bin/protoc-gen-ts_proto" \
	   --ts_proto_opt=esModuleInterop=true \
	   --ts_proto_opt=oneof=unions \
	   --ts_proto_out="./src" \
		 --experimental_allow_proto3_optional \
	   -I/usr/home/proto "/usr/home/proto/local.proto"

COPY ./figcli2/ /usr/home/figcli/
RUN cd /usr/home/figcli && make clean && BUILT_PRODUCTS_DIR=$(pwd) make all

# Set up figterm.
COPY ./figterm/ /usr/home/figterm/
RUN cd /usr/home/figterm && make clean && BUILT_PRODUCTS_DIR=${FIG_BUNDLE_EXECUTABLES} make all
RUN cp ${FIG_BUNDLE_EXECUTABLES}/figcli /usr/home/bin/figcli
RUN cp ${FIG_BUNDLE_EXECUTABLES}/figcli /usr/home/bin/fig

# Set up config repo.
COPY ./config/ /usr/home/config/
RUN mkdir -p /usr/home/config/bin/ && cp -r /usr/home/.fig_bin/* /usr/home/config/bin/
# Provide easy script for child images to call to add fig to dotfiles.
RUN echo '!#/bin/bash\ncd /usr/home/config/ && ./tools/install_and_upgrade.sh local\n' > ~/install-fig
RUN chmod +x ~/install-fig

CMD ["bash"]
