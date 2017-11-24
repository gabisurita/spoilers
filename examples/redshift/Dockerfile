FROM rustlang/rust:nightly
WORKDIR /src
COPY . /src/
RUN cargo install diesel_cli
RUN cargo install
ENTRYPOINT ["redshift_events"]
