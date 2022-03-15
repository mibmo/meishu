#!/bin/sh

# builder
builder=$(buildah from docker.io/rust:1.58.1-alpine)
buildmnt=$(buildah mount $builder)
buildah config --workingdir /builder $builder
buildah add --contextdir $PWD $builder /
buildah run $builder sh -c '
	apk add musl-dev && \
	cd /builder && \
	cargo build --release
'

# runner
runner=$(buildah from scratch)
runnermnt=$(buildah mount $runner)
cp ${buildmnt}/builder/target/release/meishu $runnermnt
buildah config --cmd "/meishu" $runner
buildah commit $runner meishu:latest

# cleanup
buildah rm $builder
buildah rm $runner
