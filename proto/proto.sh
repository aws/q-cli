#!/bin/bash
DIRECTORY=$(cd `dirname $0` && pwd)
echo $DIRECTORY

echo "Compiling fig.proto..."
protoc --plugin="$DIRECTORY/../typescript-api-bindings/node_modules/.bin/protoc-gen-ts_proto" \
	   --ts_proto_opt=esModuleInterop=true \
	   --ts_proto_opt=oneof=unions \
	   --ts_proto_out="$DIRECTORY/../typescript-api-bindings/src" \
	   --swift_opt=Visibility=Public \
	   --swift_out="$DIRECTORY/../swift-api-bindings/Sources/FigAPIBindings" \
	   "./fig.proto"

export PATH=$(go env GOPATH)/bin:$PATH

echo "Compiling local.proto..."
protoc --swift_opt=Visibility=Public \
	   --swift_out="$DIRECTORY/../fig/Local IPC" \
	   --go_opt=paths=source_relative \
	   --go_opt=Mlocal.proto="fig-proto/" \
	   --go_out="$DIRECTORY/../figcli2/fig-proto" \
	   "./local.proto"

