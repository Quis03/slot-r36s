FROM rust:1-bullseye
# bullseye is EOL and its security pool has left deb.debian.org; use the
# snapshot the image itself was built from so -dev packages match.
RUN sed -i -e 's|^deb http://deb.debian.org|# &|' -e 's|^# deb http://snapshot|deb [check-valid-until=no] http://snapshot|' \
      /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
      cmake zip pkg-config libudev-dev libasound2-dev device-tree-compiler \
 && rm -rf /var/lib/apt/lists/*
