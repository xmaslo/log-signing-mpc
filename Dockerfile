FROM rust:1.69

RUN useradd -ms /bin/bash xdavid | chpasswd && adduser xdavid sudo
USER xdavid

COPY . .

RUN cargo test && cargo install --path .
