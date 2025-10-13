#include <mutex>

#include "manager.hpp"

std::mutex test_service_manager_mutex;

extern "C" {
void launch(void) {
    const std::lock_guard<std::mutex> lock(test_service_manager_mutex);
    auto &manager = test_service::Manager::instance();
    manager.launch_test_service();
}

void terminate(void) {
    const std::lock_guard<std::mutex> lock(test_service_manager_mutex);
    auto &manager = test_service::Manager::instance();
    manager.terminate_test_service();
}
}
