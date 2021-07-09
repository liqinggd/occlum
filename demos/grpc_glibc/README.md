# Run gRPC C++ Client/Server on Occlum

## Step 1:
Download, build and install cares, protobuf and gRPC:
```
./download_and_install_grpc_glibc.sh
```

## Step 2:
Prepare the gRPC C++ Hello World sample project, which consists of a client and server:
```
./prepare_client_server_glibc.sh
```

## Step 3:
Run the demo `server` which will listen on port `50051` on occlum:
```
./run_server_on_occlum.sh
```
Then you can invoke gRPC service by running `client` in a different terminal on occlum:
```
./run_client_on_occlum.sh
```
And you will see the "Greeter received: Hello world" in the client side output.

Or you can run the stress test through:
```
./run_stress_test.sh
```
