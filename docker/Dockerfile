FROM debian:bookworm-slim
COPY docker/ksh-cli /usr/local/bin/ksh-cli
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
