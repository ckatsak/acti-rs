# syntax=docker/dockerfile:1

FROM rust:1.62-slim-bullseye AS builder
LABEL maintainer="ckatsak@gmail.com" stage=builder

ADD . /acti-rs
RUN apt-get update && apt-get -y install make pkg-config libclang-dev curl libssl-dev \
	&& cargo install bindgen \
	&& rustup component add rustfmt \
    && curl -LO https://download.open-mpi.org/release/hwloc/v2.7/hwloc-2.7.1.tar.gz \
	&& tar -xzf hwloc-2.7.1.tar.gz \
	&& cd /hwloc-2.7.1/ \
	&& ./configure --prefix=/usr --disable-cairo --disable-libxml2 --disable-io --disable-pci \
		--disable-opencl --disable-cuda --disable-nvml --disable-rsmi --disable-levelzero \
		--disable-gl --disable-libudev --disable-plugin-dlopen --disable-plugin-ltdl \
	&& make -j$(nproc) \
	&& make install \
    && cd /acti-rs \
    && cargo build --release \
    && strip -s /acti-rs/target/release/registrant


FROM gcr.io/distroless/base-debian11
LABEL maintainer="ckatsak@gmail.com"

COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/libhwloc.so.15.5.3 /usr/lib/libhwloc.so.15
COPY --from=builder /acti-rs/target/release/registrant /

CMD ["/registrant"]
