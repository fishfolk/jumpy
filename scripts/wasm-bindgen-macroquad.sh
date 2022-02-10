#!/usr/bin/env bash

set -e

HELP_STRING=$(cat <<- END
	usage: build_wasm.sh PROJECT_NAME [--release]

	Build script for combining a Macroquad project with wasm-bindgen,
	allowing integration with the greater wasm-ecosystem.

	example: build_wasm.sh flappy-bird

	This'll go through the following steps:

	    1. Build as target 'wasm32-unknown-unknown'
	    2. Create the directory 'wbindgen' if it doesn't already exist
	    3. Run wasm-bindgen with output into the wbindgen directory
	    4. Apply patches to the output js file (detailed here: https://github.com/not-fl3/macroquad/issues/212#issuecomment-835276147)

	Required arguments:

	    PROJECT_NAME            The name of the artifact/target/project

	Arguments:

	    --release               Build in release mode


	Author: Tom Solberg <me@sbg.dev>
	Version: 0.1
END
)


die () {
    echo >&2 "usage: build_wasm.sh PROJECT_NAME [--release]"
    echo >&2 "Error: $@"
    echo >&2
    exit 1
}


# Storage
RELEASE=no
POSITIONAL=()

# Parse primary commands
while [[ $# -gt 0 ]]
do
    key="$1"
    case $key in
        --release)
            RELEASE=yes
            shift
            ;;

        -h|--help)
            echo "$HELP_STRING"
            exit 0
            ;;

        *)
            POSITIONAL+=("$1")
            shift
            ;;
    esac
done

# Restore positionals
set -- "${POSITIONAL[@]}"
[ $# -ne 1 ] && die "too many arguments provided"

PROJECT_NAME=$1

TARGET_DIR="target/wasm32-unknown-unknown"

EXTRA_ARGS=""
if [ "$RELEASE" == "yes" ]; then
    EXTRA_ARGS=" --release"

    TARGET_DIR="$TARGET_DIR/release"
else
    TARGET_DIR="$TARGET_DIR/debug"
fi

HTML=$(cat <<- END
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>${PROJECT_NAME}</title>
    <style>
        html,
        body,
        canvas {
            margin: 0px;
            padding: 0px;
            width: 100%;
            height: 100%;
            overflow: hidden;
            position: absolute;
            z-index: 0;
        }
    </style>
</head>
<body>
    <canvas id="glcanvas" tabindex='1'></canvas>
    <!-- Minified and statically hosted version of https://github.com/not-fl3/macroquad/blob/master/js/mq_js_bundle.js -->
    <script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"></script>
    <script type="module">
        import init, { set_wasm } from "./${PROJECT_NAME}.js";
        async function run() {
            let wbg = await init();
            miniquad_add_plugin({
                register_plugin: (a) => (a.wbg = wbg),
                on_init: () => set_wasm(wasm_exports),
                version: "0.0.1",
                name: "wbg",
            });
            load("./${PROJECT_NAME}_bg.wasm");
        }
        run();
    </script>
</body>
</html>
END
)

# Build
cargo build --target wasm32-unknown-unknown $EXTRA_ARGS

# Generate bindgen outputs
mkdir -p web
wasm-bindgen --target web --out-dir web/ $TARGET_DIR/$PROJECT_NAME.wasm

# Shim to tie the thing together
sed -i "s/import \* as __wbg_star0 from 'env';//" web/$PROJECT_NAME.js
sed -i "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" web/$PROJECT_NAME.js
sed -i "s/imports\['env'\] = __wbg_star0;/return imports.wbg\;/" web/$PROJECT_NAME.js

echo "$HTML" > web/index.html