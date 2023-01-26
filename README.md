# jlox implemented in rust

A tree-walk interpreter implemented in Rust follow Bob Nystrom's [Crafting Interpreters](https://www.craftinginterpreters.com/).

This is the toy project I used while learning Rust. The implementation is not good and might be a little bit slow.

## Examples

#### fibonacci
- code
```
fun fib(n) {
  if (n <= 1) return n;
  return fib(n - 1) + fib(n - 2);
}

for (var i = 1; i < 10; i = i + 1) {
    print fib(i);
}
```
- run
```bash
cargo run --release -- examples/fibonacci.lox
```
result
```
1
1
2
3
5
8
13
21
34
```

#### Error handling
I tried to improve error message a little bit rather than just throw the line with error.
- code
```
class A {
    init() {
        return "error";
    }
}
```
- run
```bash
cargo run --release -- examples/return_error.lox
```
- result
```
[line 3]: ResolveError: Could not return inside constructor
        return "error";
        ^^^^^^^^^^^^^^^
```


#### Callback
- code
```
class Thing {
  getCallback() {
    fun localFunction() {
      print this.name;
    }

    return localFunction;
  }
}
var thing = Thing();
thing.name = "No name";
var callback = thing.getCallback();
callback();
```
- run
```bash
cargo run --release -- examples/callback.lox
```
- result
```
No name
```

#### REPL
- run
```bash
cargo run --release
```
```
Welcome to Lox prompt
>>> var world = "World";
>>> world
World
>>>
>>> fun hello(name) { return "Hello " + name; }
>>> hello(world)
Hello World
>>>
>>> hello("Lox")
Hello Lox
>>>
```