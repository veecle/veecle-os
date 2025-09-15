/**
 * Provides implementation of a test service
 * generated out from Franca IDL files.
 */
#ifndef _SRC_SERVICE_HPP
#define _SRC_SERVICE_HPP

#include <CommonAPI/Logger.hpp>
#include <v0/test/TestServiceStubDefault.hpp>

#include "utils.hpp"

namespace test_service {

/**
 * Provides implementation for virtual methods of
 * the service generated out from Franca IDL files.
 *
 * All methods echo input back to the sender.
 */
class TestServiceStubImpl : public v0::test::TestServiceStubDefault {
    using AllPrimitiveDataTypes = v0::test::TestService::AllPrimitiveDataTypes;
    using ClientId = CommonAPI::ClientId;

  public:
    void test_bool(const std::shared_ptr<ClientId> _client, bool _flag, test_boolReply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_flag);
    }

    void test_int8(const std::shared_ptr<ClientId> _client, int8_t _param, test_int8Reply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_int16(const std::shared_ptr<ClientId> _client, int16_t _param, test_int16Reply_t _reply) {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_int32(const std::shared_ptr<ClientId> _client, int32_t _param, test_int32Reply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }
    void test_int64(const std::shared_ptr<ClientId> _client, int64_t _param, test_int64Reply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_uint8(const std::shared_ptr<ClientId> _client, uint8_t _param, test_uint8Reply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_uint16(const std::shared_ptr<ClientId> _client, uint16_t _param, test_uint16Reply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_uint32(const std::shared_ptr<ClientId> _client, uint32_t _param, test_uint32Reply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_uint64(const std::shared_ptr<ClientId> _client, uint64_t _param, test_uint64Reply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_double(const std::shared_ptr<ClientId> _client, double _param, test_doubleReply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_float(const std::shared_ptr<ClientId> _client, float _param, test_floatReply_t _reply) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_struct(
        const std::shared_ptr<ClientId> _client, AllPrimitiveDataTypes _request, test_structReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_request);
    }

    void test_utf16le_dynamic_length_string(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::string _param,
        test_utf16le_dynamic_length_stringReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_utf16be_dynamic_length_string(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::string _param,
        test_utf16be_dynamic_length_stringReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_utf8_dynamic_length_string(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::string _param,
        test_utf8_dynamic_length_stringReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_utf16le_fixed_length_string(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::string _param,
        test_utf16le_fixed_length_stringReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_utf16be_fixed_length_string(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::string _param,
        test_utf16be_fixed_length_stringReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_utf8_fixed_length_string(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::string _param,
        test_utf8_fixed_length_stringReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_fire_and_forget_uint64(const std::shared_ptr<CommonAPI::ClientId> _client, uint64_t _param) override {
        LOG_FUNCTION_CALL();
        LOG(_param);
    }

    void test_fixed_length_array(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::vector<uint32_t> _param,
        test_fixed_length_arrayReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_dynamic_length_1_byte_array(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::vector<uint32_t> _param,
        test_dynamic_length_1_byte_arrayReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_dynamic_length_2_bytes_array(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::vector<uint32_t> _param,
        test_dynamic_length_2_bytes_arrayReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }

    void test_dynamic_length_4_bytes_array(
        const std::shared_ptr<CommonAPI::ClientId> _client,
        std::vector<uint32_t> _param,
        test_dynamic_length_4_bytes_arrayReply_t _reply
    ) override {
        LOG_FUNCTION_CALL();
        _reply(_param);
    }
};

} // namespace test_service

#endif // _SRC_SERVICE_HPP
