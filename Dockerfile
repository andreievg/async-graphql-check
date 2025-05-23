
FROM rust:latest

RUN apt-get update && apt-get install -y git make

# Install faketime
WORKDIR /usr/src/
RUN git clone https://github.com/wolfcw/libfaketime.git
WORKDIR /usr/src/libfaketime/src
RUN make install
# This will make libfaketime.so.1 run when time is requested form the system
# making FAKETIME env variable work
RUN echo "/usr/src/libfaketime/src/libfaketime.so.1" > /etc/ld.so.preload

WORKDIR /usr/src/test
# Copy executable
COPY target/release/async-graphql-check .
# Copy entry
COPY entry.sh .

WORKDIR /usr/src/test

RUN chmod +x async-graphql-check
RUN chmod +x entry.sh

ENTRYPOINT ["/usr/src/test/entry.sh"]

EXPOSE 8000