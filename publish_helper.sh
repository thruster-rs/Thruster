#!/bin/bash

DIR=`pwd`;

# Proc Macros
cd $DIR/thruster-proc;
cargo publish --allow-dirty;

# Thruster
cd $DIR/thruster;
cargo publish --allow-dirty;
