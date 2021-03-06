#!/usr/bin/env sh

: ${TARGETPLATFORM=}
: ${TARGETOS=}
: ${TARGETARCH=}

: ${OS=}
: ${ARCH=}

env

if [ -z "${TARGETPLATFORM}" ]; then
  echo "TARGETPLATFORM environment variable is not set"
  exit 1
fi

TARGETOS="$(echo ${TARGETPLATFORM} | cut -d"/" -f1)"
if [ -z "${TARGETOS}" ]; then
  echo "no operating system found in ${TARGETPLATFORM}"
  exit 1
fi


TARGETARCH="$(echo ${TARGETPLATFORM} | cut -d"/" -f2)"
if [ -z "${TARGETARCH}" ]; then
  echo "no CPU architecture found in ${TARGETPLATFORM}"
  exit 1
fi

OS="${TARGETOS}"

case "${TARGETARCH}" in
"amd64")
  ARCH="x86_64"
  ;;
"arm64")
  ARCH="aarch64"
  ;;
*)
  echo "${TARGETARCH} is not a supported CPU architecture"
  exit 1
esac

mkdir -p /binaries/plugins \
&& cp -v /usr/local/target/release/engine /binaries/engine \
&& cp -v /usr/local/target/release/client /binaries/client \
&& cp -v /usr/local/target/release/runtime /binaries/runtime \
&& cp -v /usr/local/target/release/plugin-mount /binaries/plugins/mount-${OS}-${ARCH} \
&& cp -v /usr/local/target/release/plugin-resolver /binaries/plugins/resolver-${OS}-${ARCH} \
&& find /binaries -type f -exec strip -v {} \;
