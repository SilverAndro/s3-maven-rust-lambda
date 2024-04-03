# s3 maven rust lambda

rust program that implements a maven repository on top of aws lambda
and s3 by acting as an on-demand translation layer.

---

i generally dislike rust as i find it difficult to parse, so sorry if this
code is bad, im not interested in fixing it right now as long as it continues to work. it was this
or golang cause JVMs take too long to start and c++ is way too difficult to link with openssl
on windows.

## build
build.bat or `cargo lambda build --release --output-format zip --arm64`
