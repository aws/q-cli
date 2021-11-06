mkdir -p ~/.fig/autocomplete; cd $_

# Download all the files in the specs folder of this repo
curl https://codeload.github.com/withfig/completion-specs/tar.gz/master | \
tar -xz --strip=2 completion-specs-master/specs

TEMPALIAS='function cd() { builtin cd "$1"; fig bg:cd; }\nfig bg:cd'
grep -q $TEMPALIAS ~/.fig/exports/env.sh || echo '\n\n'$TEMPALIAS'\n\n' >> ~/.fig/exports/env.sh

