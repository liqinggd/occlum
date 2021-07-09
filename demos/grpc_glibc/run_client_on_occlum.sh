#!/bin/bash
set -e

if [ ! -d "occlum_client_instance" ];then
    mkdir occlum_client_instance
    cd occlum_client_instance
    occlum init

    mkdir -p image/etc
    cp /etc/resolv.conf image/etc
    cp ../grpc/examples/cpp/helloworld/cmake/build/greeter_client image/bin
    cp /usr/local/lib/libprotobuf.so.3.10.0.0 image/opt/occlum/glibc/lib
    cp /usr/local/lib/libcares.so.2 image/opt/occlum/glibc/lib
    cp /lib/x86_64-linux-gnu/libz.so.1 image/opt/occlum/glibc/lib
    cp /opt/occlum/glibc/lib/librt.so.1 image/opt/occlum/glibc/lib
    cp /opt/occlum/glibc/lib/libresolv.so.2 image/opt/occlum/glibc/lib
    occlum build
else
    cd occlum_client_instance
fi

occlum run /bin/greeter_client
