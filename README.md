## Esper

Esper is a minimal expression-based language that targets C++.

### Features

This list is partially a feature matrix of existing features.

- [x] Non-obtrusive syntax, Esper is related to [ML](<https://en.wikipedia.org/wiki/ML_(programming_language)>)
- [x] Readable output based on matching semantics
- [x] Default no-emit mode that delegates output to `clang++`
- [x] Expression-level directives for compiler routines

### Installation & usage

_This is section is incomplete._

### Quick Overview

The table below compares Esper source programs to the corresponding C++ output (target is `EmitDefault`). In context, a `main` function definition is expected since every module is in a separate namespace.

<table><thead>
<tr>
<th>Item</th>
<th>Esper</th>
<th>C++</th>
<th>Description</th>
</tr></thead>
<tbody>

<!-- typed variable definitions -->

<tr>
<td>Typed definitions</td>
<td>

```fs
let n : int = 0

let p : 0 = 0

let t :| bool | string = true

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

</tbody>
</table>

### License

Copyright © MIT License & GPLv3 License. Esper is dual-licensed by source; however, if `EmitContextImpl::use_glibcxx` is enabled (default), it includes headers for `libstdc++`, which is GNU software.
