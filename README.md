# Stars/Forks JSON Smartmodule

SmartModule that reads a `json` records with `stars` and `forks` fields, identifies if the new values have changed, and products a `result` record. This SmartModule is `filter-map` type, and it only returns an output if it detects changes.

**Expected Input**

```
{"forks":134,"stars":1723}
{"forks":134,"stars":1722}
{"forks":134,"stars":1722}
{"forks":135,"stars":1722}
{"forks":136,"stars":1723}
...
```

**Produced Output**

```
{"result":":star2: 1722"}
{"result":":gitfork: 135"}
{"result":":gitfork: 136 \n:star2: 1723"}
...
```

**SMDK Test**

```
smdk test --file ./test-data/input.txt
```