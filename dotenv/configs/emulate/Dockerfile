FROM --platform=linux/amd64 dotenv-base

COPY zshrc ~/.zshrc

RUN ~/install-fig

ENTRYPOINT ["npm", "run", "test", "--"]
CMD ["tests/zsh"]
