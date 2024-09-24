## Esper

Esper is a minimal expression-based language that targets C++.

### Features

This list is partially a feature matrix of existing features.

- [x] Non-obtrusive syntax, Esper is related to [ML](<https://en.wikipedia.org/wiki/ML_(programming_language)>)
- [x] Readable output based on matching semantics
- [x] Default no-emit mode that delegates output to `clang++`
- [x] Expression-level directives for compiler routines

### Installation & usage

Minimum versions:

- **rustup:** 1.27.0
- **rustc:** 1.79.0-nightly
- **clang++:** 16.0.6+ (release builds available with [LLVM installation](https://releases.llvm.org/download.html))

Clone the repository, build with Cargo (`cargo build --release`), and run `esper --help`. Upstream build is tested on Debian with ELF binaries (target `x86_64-pc-linux-gnu`). On Windows, demangling issues may arise unless the prelude is excluded (WSL/MinGW should be fine).

Range expressions require `-std=c++20` when compiling the output C++ source. Optional flags are passed to `clang++` as **raw arguments**.

```sh
esper <input> -o <output> -- -std=c++20 -Wall -O3
```

### Quick Overview

The table below compares Esper source programs to the corresponding C++ output (target is `EmitDefault`). In context, a `main` function definition is expected since every module is in a separate namespace. Refer to the [tests](https://github.com/elricmann/esper/tree/main/tests).

<table><thead>
<tr>
<th>Item</th>
<th>Esper</th>
<th>C++</th>
<th>Description</th>
</tr></thead>
<tbody>

<!-- Type alias -->
<tr>
<td>Type alias</td>
<td>

```fs
type T = int end
```

</td>
<td>

```cpp
using T = int;
```

</td>
<td>

_-_

</td>
</tr>

<!-- Parametric type alias -->
<tr>
<td>Parametric type alias</td>
<td>

```fs
type T<K> = K end
```

</td>
<td>

```cpp
template<typename K>
using T = K;
```

</td>
<td>

_Type parameters are required when instantiating._

</td>
</tr>

<!-- Reference & pointer types -->
<tr>
<td>Reference & pointer types</td>
<td>

```fs
type T<K> = &K end
type P<U> = **U end
```

</td>
<td>

```cpp
template<typename K>
using T = &K;

template<typename U>
using P = **U;
```

</td>
<td>

_Type parameters are required when instantiating._

</td>
</tr>

<!-- Variant types (tagged unions) -->
<tr>
<td>Variant types (tagged unions)</td>
<td>

```fs
type N = | int | float end

type V<T, K> = | T | K end
```

</td>
<td>

```cpp
using N = std::variant<int, float>;

template<typename T, typename K>
using V = std::variant<T, K>;
```

</td>
<td>

_-_

</td>
</tr>

<!-- Optional type -->
<tr>
<td>Optional type</td>
<td>

```fs
type T = ?int end

type K =
  | ?int
  | ?bool
end

type U = ?| int | bool end
```

</td>
<td>

```cpp
using T = std::optional<int>;

using K = std::variant<std::optional<int>, std::optional<bool>>;

using U = std::optional<std::variant<int, bool>>;
```

</td>
<td>

_Alias of `std::optional`. Wraps type expressions to the right. Variant of optionals is not an optional of variant of types._

</td>
</tr>

<!-- Mapped types -->
<tr>
<td>Mapped types</td>
<td>

```fs
type M<K, V> = { key: K, value: V } end

type tree<T> = {
  value: T,
  children: vector<tree<T>>
} end
```

</td>
<td>

```cpp
template<typename K, typename V>
struct M {
  using key   = K;
  using value = V;
}

template<typename T>
struct tree {
  using value      = T;
  using children   = std::vector<tree<T>>;
}
```

</td>
<td>

_Represents structural definitions that can be passed as signatures in polymorphic functions._

</td>
</tr>

<!-- Type members -->
<tr>
<td>Type members</td>
<td>

```fs
type P<Q> = Q.key. end
```

</td>
<td>

```cpp
template<typename Q>
using P = Q::key;
```

</td>
<td>

_Overloads the `::` syntax when accessing members of type expressions._

</td>
</tr>

<!-- Type extensions -->
<tr>
<td>Type extensions</td>
<td>

```fs
@extend(S, string) type R<S> = S end
```

</td>
<td>

```cpp
template<typename S>
using R = std::enable_if_t<
  std::is_same<
    S, std::string>::value
  S
>;
```

</td>
<td>

_`@extend` modifier required to apply parametric extended types. Ensures fst extends snd or incurs an error without static assertion._

</td>
</tr>

<!-- Type-level function definition -->
<tr>
<td>Type-level function definition</td>
<td>

```fs
type F =
  |a: int, b: float| ?int end
end
```

</td>
<td>

```cpp
using F = std::function<std::optional<int>(int, int)>;
```

</td>
<td>

_Return types are parsed as `type_expr` rule, values are treated as types regardless._

</td>
</tr>

<!-- Pattern matching -->
<tr>
<td>Pattern matching</td>
<td>

```fs
match n with
| int ->
  print("-> scope");
  print("int: ", _),
| string -> print("string: ", _),
end
```

</td>
<td>

```cpp
std::visit([](auto&& _) {
  using T = std::decay_t<decltype(_)>;
  if constexpr (std::is_same_v<T, int>) {
    print("-> scope");
    print("int: ", _);
  }
  if constexpr (std::is_same_v<T, string>) {
    print("string: ", _);
  }
  }, n);
```

</td>
<td>

<i>

Non-exhaustive matching, inner values captured as the `_`symbol. Requires`std::visit`and decaying inner value to base value types. Ideally,`get_if`and`holds_alternative` are performant but not as rigorous.

</i>

</td>
</tr>

<!-- typed variable definitions -->
<tr>
<td>Typed definitions</td>
<td>

```fs
let n : int = 0

let p : 0 = 0

let t : | bool | string = true
```

</td>
<td>

```cpp
int n = 0;

decltype(0) p = 0;

std::variant<bool, std::string> t = true;
```

</td>
<td>

_`Expr::TypedSymbol` represents type identifiers. Tagged unions are variant entries. Literal types are `decltype(T)` which is a non-constraint on the rvalue._

</td>
</tr>

<!-- Parameterized type -->
<tr>
<td>Parameterized types (postfix generics)</td>
<td>

```fs
let lst : vector<int> = []
```

</td>
<td>

```cpp
std::vector<int> lst = {};
```

</td>
<td>

_-_

</td>
</tr>

<!-- typed call expressions -->
<tr>
<td>Typed call expressions (postfix generics)</td>
<td>

```fs
let lst = vector<int>()
```

</td>
<td>

```cpp
auto lst = std::vector<int>();
```

</td>
<td>

_-_

</td>
</tr>

<!-- variable definitions -->
<tr>
<td>Variable definitions</td>
<td>

```fs
let n = 0
```

</td>
<td>

```cpp
auto n = 0;
```

</td>
<td>

_Initialization of a value is expected. Default type is `auto`. Multiple definitions as `Expr::Let` is not allowed._

</td>
</tr>

<!-- function definitions -->
<tr>
<td>Function definitions</td>
<td>

```fs
let _ = || pass end (* since v0.2 *)

let add: int = |a: int, b: int| a + b end

let swap: tuple<int> = |a: int, b: int|
  let tmp = a;
  a = b;
  b = tmp;
  [a, b]
end
```

</td>
<td>

```cpp
auto _() { return; }

int add(int a, int b) { return (a + b); }

std::tuple<int> swap(a: int, b: int) {
  auto tmp = a;
  a = b;
  b = tmp;
  return {a, b};
}
```

</td>
<td>

_Required return type is the lvalue. Non-inferred parameter types. Last expression is returned. Multiline expressions are delimited with `;`._

</td>
</tr>

<!-- Struct definition -->
<tr>
<td>Struct definition</td>
<td>

```fs
struct A end

struct B
  c: float,
  d: || c end
end
```

</td>
<td>

```cpp
class A {};

class B {
public:
  float c;
  auto d() { return c; }
};
```

</td>
<td>

_All symbols are public without `@pub`. Structs are classes. Methods are fields with function rvalues._

</td>
</tr>

<!-- Loops -->
<tr>
<td>Loops</td>
<td>

```rust
for a in b [] end
for p in q.r. [] end

for i in 0..10
  print(i)
end

for [a, b] in c
  print(a, b)
end
```

</td>
<td>

```cpp
for (auto a : b) {}
for (auto p : q.r) {}

for (auto i : views::iota(0,10)) {
  print(i);
}

for (auto [a, b] : c) {
  print(a, b);
}
```

</td>
<td>

_-_

</td>
</tr>

<!-- Value operators -->
<tr>
<td>

Value operators (since `v0.2`)

</td>
<td>

```fs
~a
&a
&&a
a gt b
a lt b
a gte b
a lte b
a eq b
a neq b
a and b
a or b
a shl b
a shr b
a band b
a bor b
a xor b
a rotl b
a rotr b
```

</td>
<td>

```cpp
~a
&a
&&a
(a > b);
(a < b);
(a >= b);
(a <= b);
(a == b);
(a != b);
(a && b);
(a || b);
(a << b);
(a >> b);
(a & b);
(a | b);
(a ^ b);
__builtin_rotateleft32(a, b);
__builtin_rotateright32(a, b);
```

</td>
<td>

_-_

</td>
</tr>

</tbody>
</table>

### Postscriptum

Esper is **experimental** and aims to stay minimal. Matching semantics are not optimized, e.g `std::visit` for pattern matching is a known performance bottleneck, exclusively using STL Containers (`argv` is cast from a `const char**`), all `libstdc++` headers are included in the optional prelude, passing by value is exclusive and a wide range of impracticable error handling; with `clang++` errors being piped to `stdin`, PEG's obscure parsing errors (resolved in `v0.2`) and no type-level resolution of expressions (requires a complete type system and call graph).

### License

Copyright © 2024 Elric Neumann. MIT License.
