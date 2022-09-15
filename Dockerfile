FROM rust:1.63.0-buster as build
COPY . /kubectl-watch
WORKDIR /kubectl-watch
RUN cargo build --release

FROM debian:buster-slim as cache
RUN apt update \
    && apt install -y tini git

FROM debian:buster-slim
RUN apt update && apt install -y libpcre2-8-0 && rm -rf /var/lib/apt/lists/*
COPY --from=cache /usr/bin/git /usr/bin/git
COPY --from=cache /usr/bin/tini /usr/bin/tini
COPY --from=build /kubectl-watch/target/release/kubectl-watch /usr/local/bin/kubectl-watch

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/kubectl-watch"]
CMD ["-h"]

# FOR Local Build
# ENV HTTPS_PROXY=172.17.0.1:1081
# RUN sed -i "s@http://deb.debian.org@http://mirrors.aliyun.com@g" /etc/apt/sources.list