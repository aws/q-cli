FROM dotenv-base

# https://github.com/sindresorhus/pure
RUN npm install --global pure-prompt
COPY zshrc ~/.zshrc
RUN ~/install-fig

ENTRYPOINT ["npm", "run", "test", "--"]
CMD ["tests/zsh"]
