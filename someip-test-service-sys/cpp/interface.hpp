/**
 * This file is intended to serve as in input for bindgen
 * and provides an interface that is exposed by this library.
 */
#ifndef _INTERFACE_HPP
#define _INTERFACE_HPP

extern "C" {

/**
 * Launches the test service.
 *
 * This function blocks the calling thread until the test service is launched.
 * If the test service is already launched prior to calling this - does nothing.
 *
 * ## Thread Safety
 *
 * This function is thread-safe and can be called concurrently from multiple threads.
 *
 * ## Configuration
 *
 * Since this implementation uses Common API SOME/IP and vsomeip internally, you must set
 * the following environment variables before calling this function:
 *
 * - `COMMONAPI_CONFIG`: Path to the Common API SOME/IP ".ini" configuration file.
 * - `VSOMEIP_CONFIGURATION`: Path to the vsomeip ".json" configuration file.
 *
 * ## External Documentation
 *
 * - [Common API C++ SOME/IP
 * Guide](https://github.com/COVESA/capicxx-someip-tools/wiki/CommonAPI-C---SomeIP-in-10-minutes)
 * - [Common API C++ Configuration](https://github.com/COVESA/capicxx-core-tools/blob/master/docx/CommonAPICppUserGuide)
 * - [vsomeip Guide](https://github.com/COVESA/vsomeip/wiki/vsomeip-in-10-minutes)
 * - [vsomeip Configuration](https://github.com/COVESA/vsomeip/blob/master/documentation/vsomeipConfiguration.md)
 */
void launch(void);

/**
 * Terminates the test service.
 *
 * This function blocks the calling thread until the test service has been terminated.
 * If the test service hasn't been launched prior to calling this - does nothing.
 *
 * Note that termination means the service has been unregistered from the CommonAPI
 * runtime and has dropped any active connections. It may still perform shutdown routines
 * in the background. Therefore, avoid attempting to relaunch the test service immediately
 * after calling this function, as it may cause unexpected behavior.
 *
 */
void terminate(void);
}

#endif // _INTERFACE_HPP