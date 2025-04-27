#!/bin/bash

echo "My task ID: " $LLSUB_RANK
echo "Number of Tasks: " $LLSUB_SIZE

cargo run --bin bayesian --release -- snow2.yaml -e snow2.txt -p linear.txt -k dist.txt -t $LLSUB_RANK optimize
