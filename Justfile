template:
    #!/usr/bin/env bash
    set -euxo pipefail

    cd {{ justfile_directory() }}
    BEFORE_HASH="$(sha256sum _usage)"

    cargo run -- --help > _usage

    AFTER_HASH="$(sha256sum _usage)"

    if [ "$BEFORE_HASH" != "$AFTER_HASH" ]; then
        gomplate -f {{ justfile_directory() }}/templates/README.md -o {{ justfile_directory() }}/README.md -d data=stdin:
    fi

install:
    pre-commit install
