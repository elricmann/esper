using namespace std;

namespace emit {
auto n = 0;
auto t = true;
auto f = false;
auto pi = 3.142;
auto k = -2;
auto o = -9.8;

auto a() {
  auto b = 3;
  o = 3.1;
  t = false;
  p(2, 3, false);
  return 2;
}

auto b(c, d) {
  2;
  return c;
}
float o = 0;
decltype(0) r = 0;

int add(int a, int b) {
  return (a + b);
}
variant<bool, int> n = 2;

auto k() {
  if ((a > 2)) {
    true;
  }
  else {
    false;
  }
  fn<T>();
}
auto lst = vector<int>();
auto rng = views::iota(2, 3);
auto v = {9, 6, 3};
p.q = 3;
p.q.r = 4;
p.q.r();
p.q.r<bool>();

class A {
public:
};

class B {
public:
  int c;
  auto d(int a) {
    return (a - c);
  }
};
vector<int> p = none;

tuple<int> swap(int a, int b) {
  auto tmp = a;
  a = b;
  b = tmp;
  return {a, b};
}

for (auto a : b) {
  {};
}

for (auto p : q.r) {
  {};
}

for (auto i : views::iota(0, 10)) {
  print(i);
}

for (auto [a, b] : c) {
  print(a, b);
}

auto f(vector<int> n) {
  return n;
}
using A = b;
template<typename T> using A = T;
template<typename P, typename Q> using B = variant<P, Q>;
template<typename T, typename K> struct A {
  using key = T;
  using value = K;
};
template<typename Q> using P = Q::key;
template<typename S> using R = std::enable_if_t<std::is_same<S, string>::value, S>;
template<typename T> struct tree {
  using value = T;
  using children = vector<tree<T>>;
};
using R = optional<int>;
using S = variant<optional<int>, optional<bool>>;
using T = optional<variant<int, bool>>;
variant<int, bool> n = 0;
std::visit([](auto&& _) {
  using T = std::decay_t<decltype(_)>;
  if constexpr (std::is_same_v<T, int>) {
    print("entering");
    print("int: ", _);
  }
  if constexpr (std::is_same_v<T, string>) {
    print("string: ", _);
  }
  }, n);
using F = std::function<optional<int>(int, float)>;
using P = std::function<optional<int>(int, variant<bool, float>)>;
auto _ = (a > b);
auto _ = (a < b);
auto _ = (a >= b);
auto _ = (a <= b);
auto _ = (a == b);
auto _ = (a != b);
auto _ = (a && b);
auto _ = (a || b);
auto _ = (a << b);
auto _ = (a >> b);
auto _ = (a & b);
auto _ = (a | b);
auto _ = (a ^ b);
auto _ = __builtin_rotateleft32(a, b);
auto _ = __builtin_rotateright32(a, b);
} // namespace emit

int main(int argc, const char** argv) { return emit::main(argc, std::vector<std::string>(argv + 1, argv + argc)); }
