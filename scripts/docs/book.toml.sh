#!/bin/bash

name=`yq -y .name em.yml | head -n 1`
version=`yq -y .version em.yml | head -n 1`

m4 -PE - docs/book.toml.in > docs/book.toml << EOF
m4_changequote(<,>)m4_dnl
m4_define(S_NAME, <${name^}>)m4_dnl
m4_define(S_VERSION, <v$version>)m4_dnl
EOF
