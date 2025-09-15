/**
 * Provides a single facade to interact with test service.
 */
#ifndef _SRC_MANAGER_HPP
#define _SRC_MANAGER_HPP

#include <optional>
#include <string>
#include <thread>

#include <CommonAPI/CommonAPI.hpp>
#include <v0/test/TestServiceProxy.hpp>
#include <v0/test/TestServiceStubDefault.hpp>

#include "service.hpp"
#include "utils.hpp"

namespace test_service {

/**
 * Runtime & service configuration.
 */
namespace {

// Should be a base name (without suffix) of the library (.so)
// that has been build from files generated from Franca IDL files
// by commonapi-core and commonapi-someip generators.
auto const SOMEIP_GEN_LIBRARY_NAME_BASE = "someip-test-service";

// How much time to wait before retry when:
//    - Service (un-) registration failed.
//    - Check that service is (un-) available failed.
auto const RETRY_TIMEOUT = std::chrono::milliseconds(100);

// Identifiers used by CommonAPI SOME/IP to un-/register test service.
auto const SERVICE_DOMAIN = "local";
auto const SERVICE_INSTANCE = "test.TestService";
auto const SERVICE_INTERFACE = "test.TestService:v0_1";
auto const SERVICE_CONNECTION = "test-service";

} // namespace

/**
 * Abstracts the complexities of the test service
 * and exposes a simple API for interacting with it.
 */
class Manager {
  private:
    template <typename... _AttributeExtensions>
    using TestServiceProxy = v0::test::TestServiceProxy<_AttributeExtensions...>;

    struct Meta {
        std::string domain;
        std::string instance;
        std::string interface;
        std::string connection;
    };

    struct Service {
        Meta meta;
        std::shared_ptr<TestServiceStubImpl> handle;
    };

    std::optional<Service> running_service;

  public:
    static Manager &instance() {
        static Manager manager;
        return manager;
    }

    Manager(const Manager &) = delete;
    Manager &operator=(const Manager &) = delete;

    void launch_test_service() {
        LOG_FUNCTION_CALL();
        if (!running_service.has_value()) {
            auto service = create_service();

            register_service(service);
            wait_service(service, true);

            running_service = service;
        } else {
            LOG("Ignoring an attempt to launch - service is already launched.");
        }
    }

    void terminate_test_service() {
        LOG_FUNCTION_CALL();
        if (running_service.has_value()) {
            unregister_service(running_service.value());
            wait_service(running_service.value(), false);

            running_service.reset();
        } else {
            LOG("Ignoring an attempt to terminate - service hasn't been launched.");
        }
    }

  private:
    Manager() : running_service(std::nullopt) { configure_common_api_runtime(); }

    ~Manager() { terminate_test_service(); }

    void configure_common_api_runtime() {
        CommonAPI::Runtime::setProperty("LibraryBase", SOMEIP_GEN_LIBRARY_NAME_BASE);
        (void)CommonAPI::Runtime::get(); // Force init because we need a logger.
    }

    Service create_service() const {
        auto meta = Meta{
            .domain = SERVICE_DOMAIN,
            .instance = SERVICE_INSTANCE,
            .interface = SERVICE_INTERFACE,
            .connection = SERVICE_CONNECTION
        };
        auto handle = std::make_shared<TestServiceStubImpl>();
        return Service{.meta = meta, .handle = handle};
    }

    void register_service(const Service &service) const {
        LOG_FUNCTION_CALL();
        auto runtime = CommonAPI::Runtime::get();
        const auto &[domain, instance, _, connection] = service.meta;
        while (!runtime->registerService(domain, instance, service.handle, connection)) {
            LOG("Couldn't register service, trying again...");
            sleep();
        }
    }

    void unregister_service(const Service &service) {
        LOG_FUNCTION_CALL();
        auto runtime = CommonAPI::Runtime::get();
        const auto &[domain, instance, interface, _] = service.meta;
        while (!runtime->unregisterService(domain, interface, instance)) {
            LOG("Couldn't unregister service, trying again...");
            sleep();
        }
    }

    // Blocks the current thread until the service becomes available or unavailable.
    // Note that availability refers to an active connection with the service,
    // and does not necessarily mean that the service has been terminated.
    void wait_service(const Service &service, bool to_be_available) const {
        LOG_FUNCTION_CALL();
        auto runtime = CommonAPI::Runtime::get();
        const auto &[domain, instance, _, connection] = service.meta;
        auto proxy = runtime->buildProxy<TestServiceProxy>(domain, instance, connection);
        while (proxy->isAvailable() != to_be_available) {
            sleep();
        }
    }

    void sleep() const { std::this_thread::sleep_for(RETRY_TIMEOUT); }
};

} // namespace test_service

#endif // _SRC_MANAGER_HPP