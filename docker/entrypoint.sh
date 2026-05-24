#!/bin/sh
set -e

mkdir -p /shared

if [ "$NODE_ROLE" = "provider" ]; then
    echo "hello from node1" > /tmp/data.txt

    CID=$(ksh-cli add /tmp/data.txt)
    echo "$CID" > /shared/cid.txt
    echo "saved CID=$CID"

    # run provide in background, capture output
    ksh-cli provide "$CID" 9001 > /shared/provider_output.txt 2>&1 &
    PROVIDER_PID=$!

    sleep 2

    # extract peer id from output
    PEER_ID=$(sed -n 's/.*peer id: \([^ ]*\).*/\1/p' /shared/provider_output.txt || true)
    if [ -n "$PEER_ID" ]; then
        echo "$PEER_ID" > /shared/peer_id.txt
        echo "announced peer=$PEER_ID"
    fi

    # try to get listen addr too
    LISTEN_ADDR=$(sed -n 's/.*Listening on \([^ ]*\).*/\1/p' /shared/provider_output.txt || true)
    if [ -n "$LISTEN_ADDR" ]; then
        echo "$LISTEN_ADDR" > /shared/listen_addr.txt
        echo "listen addr=$LISTEN_ADDR"
    fi

    wait $PROVIDER_PID

elif [ "$NODE_ROLE" = "requester" ]; then
    echo "waiting for provider to be ready..."
    while [ ! -f /shared/cid.txt ] || [ ! -f /shared/peer_id.txt ]; do
        sleep 1
    done

    CID=$(cat /shared/cid.txt)
    PEER_ID=$(cat /shared/peer_id.txt)

    echo "fetcher: cid=$CID peer=$PEER_ID"

    ksh-cli fetch "$CID" "$PEER_ID" "/dns4/node1/tcp/9001"

    ksh-cli get "$CID"
    echo "DONE"
fi
