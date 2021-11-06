## Build test environments

If on an M1, run the following (or put in one of your shell rcs):
```
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_DEFAULT_PLATFORM=linux/arm64 
```

(See https://hublog.hubmed.org/archives/002027 and https://githubmemory.com/repo/docker/for-mac/issues/5873)

Run `docker-compose build dotenv-base` to build base image.
Run `docker-compose build` to build remaining images.

## Run tests

First, build the test environments. Then run `docker-compose up` to run the images.


## Create a new test environment

Assuming your environment is named `myenv`, you should:

1. Add a new folder inside of `configs` called `myenv`

2. Add a Dockerfile in this folder. A minimal example is:
```Dockerfile
FROM dotenv-base

RUN ~/install-fig

ENTRYPOINT ["npm", "run", "test", "--"]
CMD ["tests/bash", "tests/zsh"]
```
In this script you should install dependencies and set up config files
(`.zshrc`, `.bashrc`, etc.). The helper script `~/install-fig` is
available to run fig's installation script on any dotfiles you create.

Unless you have a very compelling reason, your Dockerfile should probably
use the same `FROM` and `ENTRYPOINT` directives defined in the example
above, and should define some default jest test suites with the `CMD` directive.

You can find other examples of test environment Dockerfiles in the `configs/` folder.

3. Add your environment as a new service called `myenv` in the `docker-compose.yaml`
file. As a minimal example you could have:
```yaml
  myenv:
    container_name: myenv
    build: ./configs/myenv/
    extends:
      file: common.yaml
      service: common-options
    command: tests/bash tests/zsh
```

In almost all cases, you should use the `container_name`, `build`, and `extends` values
defined above (with your environment name substituted).

The command key specifies which jest test suites you'd like to run. This
is a little redundant if you specified this in your Dockerfile with the
`CMD` directive, but is nice to include here so that all of the test
suites run across all environments can be seen in one place
(`docker-compose.yaml`). You can change this and add new test suites
specific to your environment if you'd like.

Lastly, you can use this entry to add environment variables, etc. here
or modify any other typical docker-compose service options.
