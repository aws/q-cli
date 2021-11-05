# dotenv

Testing for shell integrations and figterm

Run `DOCKER_DEFAULT_PLATFORM=linux/arm64 docker-compose build` to build
the Docker images.

The platform variable is necessary on M1's where Docker's resolution of
base image architectures is a little buggy.

Then run `docker-compose up -d` to run the images.

`cp -r configs/blank configs/<new_config_name>` to create a new set of config files
