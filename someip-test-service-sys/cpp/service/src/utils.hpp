/**
 * Provides various utilities re-used by other files.
 */
#ifndef _SRC_UTILS_HPP
#define _SRC_UTILS_HPP

#include <string>

#include <CommonAPI/Logger.hpp>

namespace test_service {

#define LOG(message) COMMONAPI_INFO(message)

/**
 * Takes input provided by __PRETTY_FUNCTION__
 * and converts it to human-redable string.
 */
inline std::string method_name(const std::string &prettyFunction) {
    size_t colons = prettyFunction.find("::");
    size_t begin = prettyFunction.substr(0, colons).rfind(" ") + 1;
    size_t end = prettyFunction.rfind("(") - begin;
    return prettyFunction.substr(begin, end) + "()";
}

#define __METHOD_NAME__ method_name(__PRETTY_FUNCTION__)

/**
 * RAII logger that prints logs when created
 * and when goes out of scope. Used to log
 * function call regardless of how control
 * flow went.
 */
class ScopedFunctionLogger {
  public:
    ScopedFunctionLogger(const std::string &method_name) : method_name_to_log(method_name) {
        LOG("[" + method_name_to_log + "] ENTER");
    }
    ~ScopedFunctionLogger() { LOG("[" + method_name_to_log + "] EXIT"); }

  private:
    const std::string method_name_to_log;
};

/**
 * Convinient macro that will print the name
 * of the function on enter and exit. Intended
 * to be placed on first line of the function.
 */
#define LOG_FUNCTION_CALL() ScopedFunctionLogger _scoped_function_logger_instance(__METHOD_NAME__)

} // namespace test_service

#endif // _SRC_UTILS_HPP
