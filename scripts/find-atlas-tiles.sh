#!/usr/bin/env bash

HERE="$(dirname "$(dirname "${BASH_SOURCE[0]}")")" || exit

find_layers() {
    yq -y -er --arg target_tilemap "$1" --arg indexes_raw "$2" '
        def filter_layer_by_tilemap:
            .tilemap // empty | endswith("/" + $target_tilemap);

        def filter_layer_by_indexes($indexes):
            .tiles as $tiles
            | $indexes
            | any(. as $i | $tiles | any(.idx == $i));

        def find_layers_with_indexes($indexes):
            .layers
            | map(select(filter_layer_by_tilemap))
            | map(select(filter_layer_by_indexes($indexes)))
            | map(.id);

        find_layers_with_indexes($indexes_raw | fromjson)
    '
}

declare LAYERS

echo
find "$HERE"/assets/map/levels -type f | sort -V | while read f; do
    LAYERS="$(find_layers "$1" "$2" < "$f")"
    [ "$LAYERS" = '[]' ] || printf '%s\n' "$f" "$LAYERS" ''
done
