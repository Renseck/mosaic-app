FROM alpine:latest
RUN apk add --no-cache curl jq bash
COPY init.sh /init.sh
RUN chmod +x /init.sh
ENTRYPOINT ["/init.sh"]