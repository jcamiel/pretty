## Data

https://microsoftedge.github.io/Demos/json-dummy-data

## Implemetations

## RFC 7159: Spec JSON

<https://datatracker.ietf.org/doc/html/rfc7159>

## Go Standard Lib JSON Decoder

<https://github.com/golang/go/blob/master/src/encoding/json/jsontext/decode.go>

## Serde JSON

<https://github.com/serde-rs/json>

## Test suite

<https://github.com/nst/JSONTestSuite>

<https://github.com/flenniken/utf8tests>



## Performance

### With just a byte reader:


```shell
$ time pretty --iter 5000 5mb.json > /dev/null        
pretty --iter 5000 5mb.json  11.25s user 0.86s system 99% cpu 12.134 total
```

With serde pretty:

```shell
$ time pretty --iter 5000 --serde 5mb.json > /dev/null
pretty --iter 5000 --serde 5mb.json  69.43s user 11.71s system 99% cpu 1:21.17 total
```

### Skipping whitespaces (simple

```shell
$ time pretty --iter 5000 5mb.json > /dev/null
pretty --iter 5000 5mb.json  35.43s user 1.10s system 98% cpu 36.914 total
```

### Validating UTF-8 before JSON parsing

```shell
$ time pretty --iter 5000 5mb.json > /dev/null
pretty --iter 5000 5mb.json  21.16s user 0.05s system 99% cpu 21.248 total
```

### Validating UFT-8 before hand + whitespace skip

```shell
$ time pretty --iter 5000 5mb.json > /dev/null
pretty --iter 5000 5mb.json  43.04s user 0.08s system 99% cpu 43.163 total
```

### Byte oriented + write in chunks + validating UTF-8 on demand

```shell
$ time target/release/pretty --serde --iter 5000 5mb.json > /dev/null
target/release/pretty --serde --iter 5000 5mb.json  69.38s user 10.02s system 99% cpu 1:19.49 total
```

```shell
$ time target/release/pretty --iter 5000 5mb.json > /dev/null 
target/release/pretty --iter 5000 5mb.json  33.77s user 2.57s system 99% cpu 36.392 total
```

## TODO

- [x] Empty array should be formatted `[]`
- [ ] Tests unit on strings
- [ ] JSON parsing suite
- [ ] Add limit on nesting
- [ ] Remove BOM
- [ ] use `[]` instead of `Vec` in `InvalidUtf8`