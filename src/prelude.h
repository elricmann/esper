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
};
}  // namespace __esper