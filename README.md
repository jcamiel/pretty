https://microsoftedge.github.io/Demos/json-dummy-data

## RFC 7159: Spec JSON

<https://datatracker.ietf.org/doc/html/rfc7159>


## Go Standard Lib JSON Decoder

<https://github.com/golang/go/blob/master/src/encoding/json/jsontext/decode.go>


## Performance

With just a byte reader:


```shell
$ time cat /Users/jc/Downloads/5MB.json | pretty --iter 5000 > /dev/null        
cat /Users/jc/Downloads/5MB.json  0.00s user 0.00s system 18% cpu 0.026 total
pretty --iter 5000 > /dev/null  11.25s user 0.86s system 99% cpu 12.134 total
```

With serde pretty:

```shell
$ time cat /Users/jc/Downloads/5MB.json | pretty --iter 5000 --serde > /dev/null
cat /Users/jc/Downloads/5MB.json  0.00s user 0.00s system 54% cpu 0.010 total
pretty --iter 5000 --serde > /dev/null  69.43s user 11.71s system 99% cpu 1:21.17 total
```



