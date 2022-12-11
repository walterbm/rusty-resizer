################################################################################
# Builder
################################################################################
FROM rust:1.65.0 as build

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN apt-get update \
     && apt-get -y install \
          build-essential \
          ca-certificates \
          clang \
          curl \
          libc-bin \
          libjpeg-turbo-progs \
          libpng-dev \
          libssl-dev \
          linux-libc-dev \
          pkg-config \
          zip \
          zlib1g-dev \
     && apt-get clean \
     && rm -rfv /var/lib/apt/lists/*

RUN update-ca-certificates

ENV IMAGE_MAGICK_VERSION 7.1

RUN curl https://imagemagick.org/archive/ImageMagick.tar.gz  | tar xz \
     && cd ImageMagick-${IMAGE_MAGICK_VERSION}* \
     && ./configure --with-magick-plus-plus=no --with-perl=no \
     && make \
     && make install \
     && cd .. \
     && rm -r ImageMagick-${IMAGE_MAGICK_VERSION}*

# Detect and link ImageMagick libraries 
RUN ldconfig

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release 

# List all the required dynamically linked libraries and copy them into a /libs directory
# to help isolate them for the next stage. 
#
# Ideally we would use a statically linked build with musl
# but ImageMagick does not work well with static builds
RUN ldd /usr/src/app/target/release/rusty_resizer | tr -s '[:blank:]' '\n' | grep '^/' | \
    xargs -I % sh -c 'mkdir -p $(dirname libs%); cp % libs%;'

################################################################################
# Final
################################################################################
FROM scratch

# Dynamically linked libraries 
COPY --from=build /usr/src/app/libs/ /usr/src/libs/

# For HTTP client requests: DNS support
COPY --from=build /lib/x86_64-linux-gnu/libnss_dns.so.2 /usr/src/libs/lib/x86_64-linux-gnu/libnss_dns.so.2
COPY --from=build /lib/x86_64-linux-gnu/libresolv.so.2 /usr/src/libs/lib/x86_64-linux-gnu/libresolv.so.2

# For HTTPS client requests: SSL support
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

# Manually specify the path for the dynamically linked libraries
ENV LD_LIBRARY_PATH=/usr/src/libs/lib/x86_64-linux-gnu:/usr/src/libs/usr/lib/x86_64-linux-gnu:/usr/src/libs/usr/local/lib

# Grab dynamically linked rust binary
COPY --from=build /usr/src/app/target/release/rusty_resizer /usr/local/bin/rusty_resizer

EXPOSE 8080

# Explicitly run the binary with dynamic linker (ld.so)
CMD ["/usr/src/libs/lib64/ld-linux-x86-64.so.2", "/usr/local/bin/rusty_resizer"]
