#!/bin/bash
# Script to clean up duplicate benchmark files

echo "Cleaning up benchmark directory..."

# Remove any existing benchmark files except integrated_benchmarks.rs
cd benches
for file in *.rs; do
    if [ "$file" != "integrated_benchmarks.rs" ]; then
        echo "Removing $file"
        rm "$file"
    fi
done

echo "Benchmark cleanup complete!"