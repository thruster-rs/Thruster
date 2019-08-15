#!/bin/bash

DIR=`pwd`;

echo $DIR
# Proc Macros
cd $DIR/thruster-proc;
cargo publish --allow-dirty;
sleep 5;

# Core async await
cd $DIR/thruster-core-async-await;
cargo publish --allow-dirty;
sleep 5;

# Core
cd $DIR/thruster-core;
cargo publish --allow-dirty;
sleep 5;

# Context
cd $DIR/thruster-context;
cargo publish --allow-dirty;
sleep 5;

# Middleware
cd $DIR/thruster-middleware;
cargo publish --allow-dirty;
sleep 5;

# Async await
cd $DIR/thruster-async-await;
cargo publish --allow-dirty;
sleep 5;

# App
cd $DIR/thruster-app;
cargo publish --allow-dirty;
sleep 5;

# Server
cd $DIR/thruster-server;
cargo publish --allow-dirty;
sleep 5;

# Thruster
cd $DIR/thruster;
cargo publish --allow-dirty;
