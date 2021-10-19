

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
cd ../typescript-api-bindings && npm install
```

##### Install go compiler
```
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest

```


##### Compile protos

```
./proto.sh
```



---

**Note**: `Package.swift` is symlinked to the root directory so that the github repo can be imported by Xcode.



- We may potentially want to host this is a private repo: https://stackoverflow.com/questions/47842479/how-to-use-swift-package-manager-with-private-repos