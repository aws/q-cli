
## Installation

##### Install protobuf compiler:

```
 brew install protobuf
```



##### Install Swift compiler

```
brew install swift-protobuf
```

##### Install TS compiler

```
npm install -g ts-proto
```

##### Install go compiler
```
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest

```

## Usage

##### Compile protos and install artifacts

```
make
```

## Contributing

##### Adding existing protos to a new project within the monorepo

Edit the Makefile:

1. Define a new destination environment variable (eg. `TYPESCRIPT_API_BINDINGS=$(ROOT)/../typescript-api-bindings/src`)

2. Add to DESTINATIONS list

3. Go to relevant project task (eg. `api:`) and copy compiled artifact to new destination

