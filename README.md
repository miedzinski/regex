# regex

Regular expressions parser written with [nom](https://github.com/Geal/nom).

# Graphviz compiler

![example NFA constructed from regex](https://raw.githubusercontent.com/miedzinski/regex/master/example.png)

There is additional tool for constructing equivalent NFA in DOT language based on algorithm shown in Dragon Book.   
The output can be piped to [Graphviz](http://www.graphviz.org), which in turn can output other formats, such as PNG.

Run with

```bash
cargo run --bin dot <<< '[0-9.[:lower:]]?|fo+(bar)?'
```

# License

GNU GPLv3.
