FROM slic/rust:alpine-edge as build
COPY . /kubectl-watch
WORKDIR /kubectl-watch
# RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories
RUN apk add --no-cache musl-dev
RUN https_proxy=http://172.17.0.1:1081 cargo build --release --target x86_64-unknown-linux-musl

FROM slic/alpine:alpine-edge
# RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories
RUN apk add --no-cache git
COPY --from=build /kubectl-watch/target/release/kubectl-watch /usr/local/bin/kubectl-watch
ENTRYPOINT "kubectl-watch"
CMD ["-h"]
