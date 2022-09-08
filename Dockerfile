FROM slic/rust:alpine-edge as build
COPY . /kubectl-watch
WORKDIR /kubectl-watch
# RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories
RUN apk add --no-cache musl-dev
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:edge
# RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories
RUN apk add --no-cache git
COPY --from=build /kubectl-watch/target/x86_64-unknown-linux-musl/release/kubectl-watch /usr/local/bin/kubectl-watch
ENTRYPOINT ["/usr/local/bin/kubectl-watch"]
CMD ["-h"]