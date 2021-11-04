FROM --platform=linux/amd64 dotenv-base

RUN git clone --recursive https://github.com/sorin-ionescu/prezto.git "$HOME/.zprezto"

ENTRYPOINT ["npm", "run", "test", "--"]
CMD ["tests/zsh"]
