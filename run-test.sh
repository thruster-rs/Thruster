#!/bin/bash

cargo run --example hello_world &> bench.log &
CURRENT_PID=$!

while ! echo exit | nc localhost 4321; do sleep 1; done
wrk -H 'Host: tfb-server' -H 'Accept: text/plain,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7' -H 'Connection: keep-alive' --latency -d 2 -c 1024 --timeout 8 -t 3 http://localhost:4321/plaintext -s /Users/pete/dev/FrameworkBenchmarks/toolset/wrk/pipeline.lua -- 16

kill -9 $CURRENT_PID

node test.js

rm bench.log
