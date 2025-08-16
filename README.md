## Data

https://microsoftedge.github.io/Demos/json-dummy-data

## Implemetations

## RFC 7159: Spec JSON

<https://datatracker.ietf.org/doc/html/rfc7159>

## Go Standard Lib JSON Decoder

<https://github.com/golang/go/blob/master/src/encoding/json/jsontext/decode.go>

## Serde JSON

<https://github.com/serde-rs/json>


## Performance

### With just a byte reader:


```shell
$ time cat 5mb.json | pretty --iter 5000 > /dev/null        
cat 5mb.json  0.00s user 0.00s system 18% cpu 0.026 total
pretty --iter 5000 > /dev/null  11.25s user 0.86s system 99% cpu 12.134 total
```

With serde pretty:

```shell
$ time cat 5mb.json | pretty --iter 5000 --serde > /dev/null
cat 5mb.json  0.00s user 0.00s system 54% cpu 0.010 total
pretty --iter 5000 --serde > /dev/null  69.43s user 11.71s system 99% cpu 1:21.17 total
```

### Skipping whitespaces (simple

```shell
$ time cat 5mb.json | pretty --iter 5000 > /dev/null
cat 5mb.json  0.00s user 0.01s system 35% cpu 0.019 total
pretty --iter 5000 > /dev/null  35.43s user 1.10s system 98% cpu 36.914 total
```

### Validating UTF-8 before JSON parsing

```shell
$ time cat 5mb.json | pretty --iter 5000 > /dev/null
cat 5mb.json  0.00s user 0.01s system 55% cpu 0.019 total
pretty --iter 5000 > /dev/null  21.16s user 0.05s system 99% cpu 21.248 total
```

### Validating UFT-8 before hand + whitespace skip

```shell
$ time cat 5mb.json | pretty --iter 5000 > /dev/null
cat 5mb.json  0.00s user 0.01s system 61% cpu 0.016 total
pretty --iter 5000 > /dev/null  43.04s user 0.08s system 99% cpu 43.163 total
```
