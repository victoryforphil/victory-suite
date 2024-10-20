```
protoc --proto_path=../admin-proto pubsub_admin.proto --grpc-web_out=import_style=typescript,mode=grpcwebtext:src --js_out=import_style=commonjs,binary:src
```