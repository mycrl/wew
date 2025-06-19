//
//  webview.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "webview.h"
#include "app.h"

#ifdef MACOS
#include "include/wrapper/cef_library_loader.h"
#endif

typedef struct
{
    CefRefPtr<IPage> ref;
} Page;

CefMainArgs get_main_args(int argc, const char** argv)
{
#ifdef WIN32
    CefMainArgs main_args(::GetModuleHandleW(nullptr));
#else
    CefMainArgs main_args(argc, const_cast<char**>(argv));
#endif
    
    return main_args;
}

void run_message_loop()
{
    CefRunMessageLoop();
}

void quit_message_loop()
{
    CefQuitMessageLoop();
}

void poll_message_loop()
{
    CefDoMessageLoopWork();
}

int execute_subprocess(int argc, const char** argv)
{
#ifdef MACOS
    CefScopedLibraryLoader library_loader;
    if (!library_loader.LoadInHelper())
    {
        return -1;
    }
#endif
    auto main_args = get_main_args(argc, argv);
    return CefExecuteProcess(main_args, new IRenderApp, nullptr);
}

void* create_app(const AppOptions* settings, AppObserver observer, void* ctx)
{
#ifdef MACOS
    CefScopedLibraryLoader library_loader;
    if (!library_loader.LoadInMain())
    {
        return nullptr;
    }
#endif
    
    assert(settings != nullptr);
    
    App* app = new App {new IApp(settings, observer, ctx)};
    return app;
}

void execute_app(void* ptr, int argc, const char** argv) {
    assert(ptr != nullptr);
    
    auto app = static_cast<App*>(ptr);
    auto main_args = get_main_args(argc, argv);
    CefExecuteProcess(main_args, app->ref, nullptr);
    CefInitialize(main_args, app->ref->cef_settings, app->ref, nullptr);
}

void close_app(void* ptr)
{
    assert(ptr != nullptr);
    
    CefShutdown();
    
    delete static_cast<App*>(ptr);
}

void* create_page(void* ptr,
                  const char* url,
                  const PageOptions* settings,
                  PageObserver observer,
                  void* ctx)
{
    assert(ptr != nullptr);
    assert(settings != nullptr);
    
    Page* page = new Page{static_cast<App*>(ptr)->ref->CreatePage(std::string(url),
                                                                  settings,
                                                                  observer,
                                                                  ctx)};
    return page;
}

void close_page(void* ptr)
{
    assert(ptr != nullptr);
    
    auto page = static_cast<Page*>(ptr);
    page->ref->IClose();
    delete page;
}

void page_send_mouse_click(void* ptr, MouseButtons button, bool pressed)
{
    assert(ptr != nullptr);
    
    static_cast<Page*>(ptr)->ref->OnMouseClick(button, pressed);
}

void page_send_mouse_click_with_pos(void* ptr,
                                    MouseButtons button,
                                    bool pressed,
                                    int x,
                                    int y)
{
    assert(ptr != nullptr);
    
    static_cast<Page*>(ptr)->ref->OnMouseClickWithPosition(button, x, y, pressed);
}

void page_send_mouse_wheel(void* ptr, int x, int y)
{
    assert(ptr != nullptr);
    
    static_cast<Page*>(ptr)->ref->OnMouseWheel(x, y);
}

void page_send_mouse_move(void* ptr, int x, int y)
{
    assert(ptr != nullptr);
    
    static_cast<Page*>(ptr)->ref->OnMouseMove(x, y);
}

void page_send_keyboard(void* ptr, int scan_code, bool pressed, Modifiers modifiers)
{
    assert(ptr != nullptr);
    
    static_cast<Page*>(ptr)->ref->OnKeyboard(scan_code, pressed, modifiers);
}

void page_send_touch(void* ptr,
                     int id,
                     int x,
                     int y,
                     TouchEventType type,
                     TouchPointerType pointer_type)
{
    assert(ptr != nullptr);
    
    // TouchEventType have the same value with cef_touch_event_type_t.
    // Same as TouchPointerType.
    static_cast<Page*>(ptr)->ref->OnTouch(id,
                                          x,
                                          y,
                                          static_cast<cef_touch_event_type_t>(type),
                                          static_cast<cef_pointer_type_t>(pointer_type));
}

void page_send_message(void* ptr, const char* message)
{
    assert(ptr != nullptr);
    assert(message != nullptr);
    
    static_cast<Page*>(ptr)->ref->ISendMessage(std::string(message));
}

void page_set_devtools_state(void* ptr, bool is_open)
{
    assert(ptr != nullptr);
    
    static_cast<Page*>(ptr)->ref->SetDevToolsOpenState(is_open);
}

void page_resize(void* ptr, int width, int height)
{
    assert(ptr != nullptr);
    
    static_cast<Page*>(ptr)->ref->Resize(width, height);
}

const void* page_get_hwnd(void* ptr)
{
    assert(ptr != nullptr);
    
    return static_cast<Page*>(ptr)->ref->GetWindowHandle();
}

void page_send_ime_composition(void* ptr, const char* input)
{
    assert(ptr != nullptr);
    assert(input != nullptr);
    
    static_cast<Page*>(ptr)->ref->OnIMEComposition(std::string(input));
}

void page_send_ime_set_composition(void* ptr, const char* input, int x, int y)
{
    assert(ptr != nullptr);
    assert(input != nullptr);
    
    static_cast<Page*>(ptr)->ref->OnIMESetComposition(std::string(input), x, y);
}
