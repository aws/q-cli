# dotenv

Testing for shell integrations and figterm

Run `docker build -t {image tag} -f ./Dockerfile ..` to build the Docker
image.

You should rebuild the image whenever you make changes to any dependencies
(including npm dependencies, and your local figterm or config folders).

Run `docker run -it -v "$(pwd):/usr/home/app/" -v /usr/home/app/node_modules {image tag}`
from this directory to run tests. The `-v` flags will mount the current
directory so that you don't need to rebuild if you are just changing js
code here.

`cp -r configs/blank configs/<new_config_name>` to create a new set of config files
