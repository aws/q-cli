protoc --plugin=./ts/node_modules/.bin/protoc-gen-ts_proto --ts_proto_opt=esModuleInterop=true --ts_proto_opt=oneof=unions --ts_proto_out=./ts/src ./fig.proto
#--swift_out=./swift/Sources 

