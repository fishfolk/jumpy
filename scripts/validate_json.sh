#!/usr/bin/env bash

# Validates the JSON files using JSON schemas (Draft-07).
# https://json-schema.org/draft-07/json-schema-release-notes.html
#
# - "file.json" is being validated if "file.schema.json" is found.
# - All of the JSON files in the same directory are being validated if "schema.json" is found.
#
# Required tool for validation: https://github.com/ajv-validator/ajv-cli/
# Example JSON schema generator: https://extendsclass.com/json-schema-validator.html

set -e

find . -name "*schema.json" -print0 | while read -rd $'\0' schema
do
    echo "[~] Schema found: $schema"
    if [[ $(basename "$schema") != "schema.json" ]]; then
        ajv validate --spec=draft7 -s "$schema" -d "${schema//.schema/}"
    else
        find "$(dirname "$schema")" -maxdepth 1 \
            -name "*.json" -not -name "*schema.json" -print0 |
                xargs -0 -I {} ajv validate --spec=draft7 -s "$schema" -d {}
    fi
done
