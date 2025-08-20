#!/bin/sh

set -e

RELEASE_TYPE=${RELEASE_TYPE:-minor}
cargo set-version --bump ${RELEASE_TYPE}
VERSION=`cargo pkgid | cut -d"#" -f2`
export MAJOR_VERSION=`echo ${VERSION} | cut -d"." -f1,2`
if [ "${RELEASE_TYPE}" != "patch" ]; then
fi
git add .
git commit -m"version ${VERSION}"
git tag v${VERSION}
git push && git push --tag