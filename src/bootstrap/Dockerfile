FROM alpine:latest
RUN apk add --no-cache curl jq bash
COPY init.sh /init.sh
COPY portal-init.sh /portal-init.sh
RUN chmod +x /init.sh /portal-init.sh
ENTRYPOINT ["/init.sh"]