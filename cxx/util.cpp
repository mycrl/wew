//
//  util.cpp
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#include "util.h"

CefMainArgs get_main_args(int argc, const char **argv)
{
#ifdef WIN32
    CefMainArgs main_args(::GetModuleHandleW(nullptr));
#else
    CefMainArgs main_args(argc, const_cast<char **>(argv));
#endif

    return main_args;
}
