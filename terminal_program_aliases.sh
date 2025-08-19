#!/usr/bin/env bash


# TODO - create a _get_binfile function in programming.sh, and reduce this array to a project list
declare -A BINARIES=(
    ['table']='table_formatter/target/release/table_formatter'
    # ['clicker']='clicker/target/release/clicker'
)

binsource_file="$(dirname $(realpath "$BASH_SOURCE"))"  # get path (without shortcuts and .. and such), get containing dir

for k in "${!BINARIES[@]}"; do
    binfile="$binsource_file/${BINARIES[$k]}"
    echo "[ -f '$binfile' ] || ( cd '${binfile%/target*}' && compile )"  # create the binaries if they're not there. Requires a prior sourcing of the `compile` function 
    echo "[ -f '$binfile' ] && function $k () { '$binfile' \"\$@\"; }"
done

