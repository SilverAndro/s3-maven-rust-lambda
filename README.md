# s3 maven rust lambda

rust lambda function that implements a maven repository on top of aws lambda
and s3 by acting as an on-demand translation layer.

---

i generally really dislike rust as i find it difficult to parse, so sorry if this
code is bad, im really not interested in fixing it as long as it works. it was this
or golang cause JVMs take too long to start and c++ is hellish to link with openssl
on windows.

## build
build.bat or `cargo lambda build --release --output-format zip --arm64`