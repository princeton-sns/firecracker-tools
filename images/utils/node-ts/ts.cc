#include <napi.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>

void ts(const Napi::CallbackInfo& info) {
    int magic = info[0].As<Napi::Number>().Uint32Value();
    int port = info[1].As<Napi::Number>().Uint32Value();
    __asm__ __volatile__("outl %0, %1"
    :
    : "a" ((uint32_t)magic),
      "d" ((uint16_t)port)
    : "memory");
}

Napi::Object init(Napi::Env env, Napi::Object exports) {
    exports.Set(Napi::String::New(env, "ts"), Napi::Function::New(env, ts));
    return exports;
};

NODE_API_MODULE(ts, init)

