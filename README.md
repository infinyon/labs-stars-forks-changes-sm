# Stars/Forks JSON Smartmodule

SmartModule that reads a `json` records with `stars` and `forks` fields, identifies if the new values have changed, and products a `result` record. This SmartModule is [filter-map] type, where a record-in may be dropped, or transformed and passed along to record-out.

### Expected Input

```
{"forks":134,"stars":1723}
{"forks":134,"stars":1722}
{"forks":134,"stars":1722}
{"forks":135,"stars":1722}
{"forks":136,"stars":1723}
...
```

### Produced Output

```
{"result":":flags: 134 \n:star2: 1723"}
{"result":":star2: 1722"}
{"result":":flags: 135"}
{"result":":flags: 136 \n:star2: 1723"}
...
```


### SMDK Compatible

This project works with `smdk` command tool:

```
smdk build
```

Run test:

```
$ smdk test --file ./test-data/input.txt
{"result":":flags: 134 \n:star2: 1723"}
{"result":":star2: 1722"}
{"result":":flags: 135"}
{"result":":flags: 136 \n:star2: 1723"}
{"result":":star2: 1724"}
```

Run test with lookback initalization:

```
$ smdk test --file ./test-data/input.txt --lookback-last 1 --record '{"forks":134,"stars":1723}'
{"result":":star2: 1722"}
{"result":":flags: 135"}
{"result":":flags: 136 \n:star2: 1723"}
{"result":":star2: 1724"}
```

Checkout Smartmodule [lookback] for additional details.


### Connector (Sink) Compatible

This smartmodule may be used inside a sink connector with the following parameter:

```
transforms:
  - uses: infinyon-labs/stars-forks-changes@0.1.3
    lookback:
      last: 1
```

[filter-map]: https://www.fluvio.io/smartmodules/transform/filter-map/
[lookback]: https://fluvio.io/smartmodules/lookback/

