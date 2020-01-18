#!/bin/bash

DIR=`pwd`;

echo $DIR
# Proc Macros
cd $DIR/thruster-proc;
cargo publish --allow-dirty;
sleep 60;

# Core async await
cd $DIR/thruster-core-async-await;
cargo publish --allow-dirty;
sleep 60;

# Core
cd $DIR/thruster-core;
cargo publish --allow-dirty;
sleep 60;

# Middleware
cd $DIR/thruster-middleware;
cargo publish --allow-dirty;
sleep 60;

# Context
cd $DIR/thruster-context;
cargo publish --allow-dirty;
sleep 60;

# Async await
cd $DIR/thruster-async-await;
cargo publish --allow-dirty;
sleep 60;

# App
cd $DIR/thruster-app;
cargo publish --allow-dirty;
sleep 60;

# Server
cd $DIR/thruster-server;
cargo publish --allow-dirty;
sleep 60;

# Thruster
cd $DIR/thruster;
cargo publish --allow-dirty;
