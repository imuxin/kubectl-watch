FROM rust:alpine3.15 as build
COPY . /kubectl-watch
WORKDIR /kubectl-watch
# RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories
RUN apk add --no-cache musl-dev openssl-dev pkgconf
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.15
# RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories
RUN apk add --no-cache git
COPY --from=build /kubectl-watch/target/release/kubectl-watch /usr/local/bin/kubectl-watch
ENTRYPOINT "kubectl-watch"
CMD ["-h"]