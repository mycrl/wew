//
//  util.h
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifndef util_h
#define util_h
#pragma once

#include "include/cef_app.h"

// clang-format off
#define IMPLEMENT_RUNNING \
  private: \
    bool _is_running = true;

#define CHECK_REFCOUNTING(result) \
    if (!_is_running) \
    { \
        return result; \
    }

#define CLOSE_RUNNING \
    _is_running = false;

// clang-format on

CefMainArgs get_main_args(int argc, const char **argv);

#endif /* util_h */
