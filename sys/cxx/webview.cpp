//
//  webview.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "webview.h"
#include "app.h"

typedef struct
{
    CefRefPtr<IBrowser> ref;
} Browser;

CefMainArgs get_main_args(int argc, const char** argv)
{
#ifdef WIN32
    CefMainArgs main_args(::GetModuleHandleW(nullptr));
#else
    CefMainArgs main_args(argc, (char**)argv);
#endif

    return main_args;
}

void execute_sub_process(int argc, const char** argv)
{
    auto main_args = get_main_args(argc, argv);
    CefExecuteProcess(main_args, new IRenderApp, nullptr);
}

void* create_webview(const WebviewOptions* settings, CreateWebviewCallback callback, void* ctx)
{
    assert(settings);
    assert(callback);

    App* app = new App;
    app->ref = new IApp(settings, callback, ctx);
    return app;
}

int webview_run(void* app_ptr, int argc, const char** argv)
{
    assert(app_ptr);

    auto app = (App*)app_ptr;

    auto main_args = get_main_args(argc, argv);
    CefExecuteProcess(main_args, app->ref, nullptr);

    if (!CefInitialize(main_args, app->ref->cef_settings, app->ref, nullptr))
    {
        return -1;
    }

#ifdef MACOS
    CefRunMessageLoop();
#endif
    return 0;
}

void webview_exit(void* app_ptr)
{
    auto app = (App*)app_ptr;

    assert(app);

#ifdef MACOS
    CefQuitMessageLoop();
#endif
    CefShutdown();
    delete app;
}

void* create_page(void* app_ptr,
                  const char* url,
                  const PageOptions* settings,
                  PageObserver observer,
                  void* ctx)
{
    assert(app_ptr);
    assert(settings);

    auto app = (App*)app_ptr;

    Browser* browser = new Browser;
    browser->ref = app->ref->CreateBrowser(std::string(url), settings, observer, ctx);
    return browser;
}

void page_exit(void* browser)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->IClose();
    delete page;
}

void page_send_mouse_click(void* browser, MouseButtons button, bool pressed)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnMouseClick(button, pressed);
}

void page_send_mouse_click_with_pos(void* browser,
                                    MouseButtons button,
                                    bool pressed,
                                    int x,
                                    int y)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnMouseClickWithPosition(button, x, y, pressed);
}

void page_send_mouse_wheel(void* browser, int x, int y)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnMouseWheel(x, y);
}

void page_send_mouse_move(void* browser, int x, int y)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnMouseMove(x, y);
}

void page_send_keyboard(void* browser, int scan_code, bool pressed, Modifiers modifiers)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->OnKeyboard(scan_code, pressed, modifiers);
}

void page_send_touch(void* browser,
                     int id,
                     int x,
                     int y,
                     TouchEventType type,
                     TouchPointerType pointer_type)
{
    assert(browser);

    auto page = (Browser*)browser;

    // TouchEventType have the same value with cef_touch_event_type_t.
    // Same as TouchPointerType.
    page->ref->OnTouch(id, x, y, (cef_touch_event_type_t)type, (cef_pointer_type_t)pointer_type);
}

void page_send_message(void* browser, const char* message)
{
    assert(browser);
    assert(message);

    auto page = (Browser*)browser;

    page->ref->ISendMessage(std::string(message));
}

void page_set_devtools_state(void* browser, bool is_open)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->SetDevToolsOpenState(is_open);
}

void page_resize(void* browser, int width, int height)
{
    assert(browser);

    auto page = (Browser*)browser;

    page->ref->Resize(width, height);
}

const void* page_get_hwnd(void* browser)
{
    assert(browser);

    auto page = (Browser*)browser;

    auto hwnd = page->ref->GetHWND();
    return (void*)hwnd;
}

void page_send_ime_composition(void* browser, const char* input)
{
    assert(browser);
    assert(input);

    auto page = (Browser*)browser;

    page->ref->OnIMEComposition(std::string(input));
}

void page_send_ime_set_composition(void* browser, const char* input, int x, int y)
{
    assert(browser);
    assert(input);

    auto page = (Browser*)browser;

    page->ref->OnIMESetComposition(std::string(input), x, y);
}
