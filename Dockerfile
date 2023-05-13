FROM rust:1.69

RUN useradd -ms /bin/bash xdavid
USER xdavid

COPY . .

RUN sudo cargo install --path .

CMD ["log-signing-mpc"]
