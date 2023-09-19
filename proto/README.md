# Proto

We use [protocol buffers](https://developers.google.com/protocol-buffers/) as a message format for inter process communication.

This folder defines three main protocols:

1. `local.proto` - Protocol for communication from local processes like
`figterm` and the `fig` CLI to the desktop app
2. `fig.proto` - Protocol for communication between client Fig.js apps
like autocomplete and the desktop app
2. `figterm.proto` - Protocol for sending commands from the desktop app to `figterm` (e.g. insert text)

## Setup

For any client, you must install the protobuf compiler:
```
 brew install protobuf
```

**Client Installations**

|Client|Command|
|---|---|
|swift|`brew install swift-protobuf`|
|typescript|`yarn install`|
|rust|N/A*|

\* The rust build process handles the installation of the proto toolchain.

## Installation/Usage

To compile protos and install artifacts, run:
```
make
```

## Deprecating a Fig API

1. Edit `fig.proto` and add the `[deprecated=true]` annotation to the relevant fields
2. Add an inline comment specifying the version when this was changed applies using the following format: `//deprecated: v1.0.53`

## Contributing

**Adding to protos**

Just edit the appropriate proto file.

**Adding a new client**

Edit the Makefile:

1. Define a new destination environment variable (eg. `TYPESCRIPT_API_BINDINGS=$(ROOT)/../typescript-api-bindings/src`)
2. Add to DESTINATIONS list
3. Go to relevant project task (eg. `api:`) and copy compiled artifact to new destination

