#include <bits/stdc++.h>
#include <cxxabi.h>

using namespace std;
namespace __esper {}
using namespace __esper;

namespace __esper {

/**
 * @brief reference types are not supported but are forwarded
 *        with ref<T>, values passed are mutable by reference
 */
// clang-format off
template <typename T> using ref = T &;
// clang-format on

/**
 * @brief requires T to equal U which returns T if true or U if false
 */
template <typename T, typename U, typename V>
using req = typename std::conditional<std::is_same<T, U>::value, U, V>::type;

/**
 * @brief unwraps a shared pointer until the inner-most non-shared
 *        value is found from a constructed type of std::shared_ptr
 */
template <typename T>
struct unwrap_t {
  using Type = T;
};

template <typename T>
struct unwrap_t<std::shared_ptr<T>> {
  using Type = typename unwrap_t<T>::Type;
};

template <typename T>
using unwrap = typename unwrap_t<T>::Type;

/**
 * @brief describes a list of types and associated properties,
 *        in this case, statically finding the length of the list
 */
template <typename... Ts>
struct type_list {};

template <typename List>
struct length_t;

template <typename... Ts>
struct length_t<type_list<Ts...>> {
  static constexpr std::size_t value = sizeof...(Ts);
};

template <typename L>
constexpr std::size_t length = length_t<L>::value;

/**
 * @brief allows asserting whether a type parameter is a container type
 *        as a SFINAE pattern (substitution failure is not an error)
 */
template <typename T, typename = void>
struct is_container_t : std::false_type {};

template <typename T>
struct is_container_t<T,
                      std::void_t<typename T::value_type, typename T::iterator>>
    : std::true_type {};

template <typename T>
inline constexpr bool is_container = is_container_t<T>::value;

/**
 * @brief decay's a likely-reference type to it's value type, removes
 *        const/volatile qualifiers and casts arrays to pointers
 */
template <typename T>
struct decay_t {
  using Type =
      typename std::remove_cv<typename std::remove_reference<T>::type>::type;
};

template <typename T>
using decay = typename decay_t<T>::Type;

/**
 * @brief dereferences reference types, deref_t is overloaded to avoid
 *        more general type erasure features from std
 */
template <typename T>
struct deref_t {
  using Type = T;
};

template <typename T>
struct deref_t<T &> {
  using Type = T;
};

template <typename T>
struct deref_t<T &&> {
  using Type = T;
};

template <typename T>
using deref = typename deref_t<T>::Type;

/**
 * @brief casts a value type to a pointer type. ptr<T> casts to a
 *        single pointer type whereas ptr_t<T> is variadic. we do
 *        this to avoid having to use pointer-like syntax in esper.
 *        however, the syntax itself will still be required when
 *        changing values in the context of the program itself
 */
template <typename T, unsigned N>
struct ptr_t_impl {
  using type = typename ptr_t_impl<T *, N - 1>::type;
};

// ptr_t_impl is a recursive template to add N pointers
// which requires a base case overload when when N = 0
template <typename T>
struct ptr_t_impl<T, 0> {
  using type = T;
};

template <typename T, unsigned N>  // alias for ptr_t_impl with N
using ptr_t = typename ptr_t_impl<T, N>::type;

// add a single ptr
template <typename T>
using ptr = T *;

/**
 * @class __esper main class for holding function definitions
 * @brief static methods on __esper are used as to avoid the :: syntax
 */
class __esper;
extern __esper esper;

class __esper {
 public:
  template <typename T>
  static std::string type_of(const T &value) {
    const char *mangled_name = typeid(value).name();
    int status = -1;

    /* check if demangling is supported by __cxa_demangle */
    char *demangled_name =
        abi::__cxa_demangle(mangled_name, nullptr, nullptr, &status);

    if (status == 0 && demangled_name != nullptr) {
      std::string demangled(demangled_name);
      std::free(demangled_name);

      return demangled;
    }

    return std::string(mangled_name);
  }

  /**
   * @brief wrapper over std::cout with variadic type args
   */
  void print() {}

  template <typename T, typename... Args>
  void print(const T &fst, const Args &...rst) {
    std::cout << fst;

    if constexpr (sizeof...(rst) > 0) {
      std::cout << " ";
    }

    print(rst...);
  }
};
}  // namespace __esper
