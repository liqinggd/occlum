#!/bin/sh
GRPC_SRC_DIR=$PWD/grpc

git clone https://github.com/grpc/grpc.git
cd grpc
git checkout tags/v1.24.3
if [ $? -ne 0 ]
then
  echo "git clone failed"
  exit 1
fi

# Install c-ares
cd $GRPC_SRC_DIR/third_party/cares/cares
git submodule update --init .
git checkout tags/cares-1_15_0
mkdir -p build
cd build
cmake ../ \
	-DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_FLAGS="-fPIC -pie" -DCMAKE_C_FLAGS="-fPIC -pie"
if [ $? -ne 0 ]
then
  echo "cares cmake failed"
  exit 1
fi
make -j$(nproc)
if [ $? -ne 0 ]
then
  echo "cares make failed"
  exit 1
fi
make install

# Install protobuf
cd $GRPC_SRC_DIR/third_party/protobuf
git submodule update --init .
git checkout tags/v3.10.0
cd cmake
mkdir -p build
cd build
cmake ../ \
	-Dprotobuf_BUILD_TESTS=OFF -DBUILD_SHARED_LIBS=TRUE \
	-DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_FLAGS="-fPIC -pie" -DCMAKE_C_FLAGS="-fPIC -pie" \
	-DCMAKE_NO_SYSTEM_FROM_IMPORTED=TRUE

if [ $? -ne 0 ]
then
  echo "protobuf cmake failed"
  exit 1
fi

make -j$(nproc)
if [ $? -ne 0 ]
then
  echo "protobuf make failed"
  exit 1
fi
make install

# Install gRPC
cd $GRPC_SRC_DIR/cmake
mkdir -p build
cd build
cmake ../.. \
	-DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_FLAGS="-fPIC -pie" -DCMAKE_C_FLAGS="-fPIC -pie" \
	-DgRPC_INSTALL=ON -DgRPC_PROTOBUF_PROVIDER=package \
	-DgRPC_ZLIB_PROVIDER=package -DgRPC_CARES_PROVIDER=package \
	-DgRPC_SSL_PROVIDER=package -DCMAKE_NO_SYSTEM_FROM_IMPORTED=TRUE

if [ $? -ne 0 ]
then
  echo "grpc cmake failed"
  exit 1
fi

make -j$(nproc)
if [ $? -ne 0 ]
then
  echo "grpc make failed"
  exit 1
fi
make install
echo "gRPC build success"
