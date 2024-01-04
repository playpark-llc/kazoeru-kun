FROM alpine:3.19.0
RUN apk add curl --no-cache
COPY ./crontabs/root /var/spool/cron/crontabs/root
CMD ["crond","-l","2","-f"]
