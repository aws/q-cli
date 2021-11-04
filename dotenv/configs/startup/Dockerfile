FROM --platform=linux/amd64 dotenv-base

RUN ~/install-fig

RUN mkdir -p ~/withfig ~/withoutfig
RUN mv ~/.bashrc ~/.zshrc ~/withfig

# Copy configs without fig from this dir
COPY bashrc ~/withoutfig/.bashrc
COPY zshrc ~/withoutfig/.zshrc

ENTRYPOINT ["npm", "run", "test", "--"]
CMD ["tests/startup"]
