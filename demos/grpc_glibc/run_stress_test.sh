#!/bin/bash
set -e

COUNT=1
while sleep 1;
do
if ./run_client_on_occlum.sh 2>&1 | grep -q 'RPC failed'; then
    echo "RPC failed at the ${COUNT}th call"
    exit 1
else
    echo "RPC success to call ${COUNT} times"
    COUNT=$[${COUNT} + 1];
fi
done
