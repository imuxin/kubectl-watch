FROM rust:1.63.0-alpine3.16 as build
COPY . /kubectl-watch
WORKDIR /kubectl-watch
# RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories
RUN apk add --no-cache musl-dev g++
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.16
# RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories
RUN apk add --no-cache git tini libstdc++
COPY --from=build /kubectl-watch/target/x86_64-unknown-linux-musl/release/kubectl-watch /usr/local/bin/kubectl-watch
ENTRYPOINT ["/sbin/tini", "--", "/usr/local/bin/kubectl-watch"]
CMD ["-h"]
