FROM alpine:3.19

# Install required packages
RUN apk add curl --no-cache

# Create scripts directory
RUN mkdir -p /scripts

# Copy and set permissions for cron scripts
COPY ./cron-scripts.sh /scripts/cron-scripts.sh
RUN chmod +x /scripts/cron-scripts.sh

# Copy crontab configuration
COPY ./crontabs/root /var/spool/cron/crontabs/root

# Run crond in foreground with verbose logging
CMD ["crond", "-l", "8", "-f"]
