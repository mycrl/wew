//
//  control.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "control.h"

#ifdef WIN32
#include "windows.h"
#endif

#define ENTER_CODE 28
#define SPACE_CODE 57
#define L_SHIFT_CODE 42
#define R_SHIFT_CODE 54
#define L_CTRL_CODE 29

#define IS_ABC(x) (x >= 0x10 && x <= 0x37)
#define IS_NUMBER(x) (x >= 2 && x <= 11)
#define IS_ALLOW_CHAR_MODE(x) (x != L_SHIFT_CODE && x != R_SHIFT_CODE && x != L_CTRL_CODE)

static const std::map<int, std::pair<char, char>> SYMBOL_REFS = {
    {2, {'1', '!'}},   {3, {'2', '@'}},  {4, {'3', '#'}},   {5, {'4', '$'}},  {6, {'5', '%'}},
    {7, {'6', '^'}},   {8, {'7', '&'}},  {9, {'8', '*'}},   {10, {'9', '('}}, {11, {'0', ')'}},
    {12, {'-', '_'}},  {13, {'=', '+'}}, {26, {'[', '{'}},  {27, {']', '}'}}, {39, {';', ':'}},
    {40, {'\'', '"'}}, {41, {'`', '~'}}, {43, {'\\', '|'}}, {51, {',', '<'}}, {52, {'.', '>'}},
    {53, {'/', '?'}} };

static bool is_symbol_from_code(int scan_code)
{
    constexpr std::array<int, 11> codes = { 26, 27, 43, 39, 40, 51, 52, 53, 41, 12, 13 };
    for (int code : codes)
    {
        if (code == scan_code)
        {
            return true;
        }
    }

    return false;
}

static std::optional<std::pair<char, char>> get_symbol_ref(int scan_code)
{
    auto iter = SYMBOL_REFS.find(scan_code);
    return iter != SYMBOL_REFS.end() ? std::optional(iter->second) : std::nullopt;
}

CefBrowserHost::MouseButtonType from_c(MouseButtons button)
{
    if (button == MouseButtons::kLeft)
    {
        return CefBrowserHost::MouseButtonType::MBT_LEFT;
    }
    else if (button == MouseButtons::kRight)
    {
        return CefBrowserHost::MouseButtonType::MBT_RIGHT;
    }
    else
    {
        return CefBrowserHost::MouseButtonType::MBT_MIDDLE;
    }
}

cef_key_event_type_t from_c(bool pressed)
{
    if (pressed)
    {
        return cef_key_event_type_t::KEYEVENT_KEYDOWN;
    }
    else
    {
        return cef_key_event_type_t::KEYEVENT_KEYUP;
    }
}

cef_event_flags_t from_c(Modifiers modifiers)
{
    if (modifiers == Modifiers::kShift)
    {
        return cef_event_flags_t::EVENTFLAG_SHIFT_DOWN;
    }
    else if (modifiers == Modifiers::kCtrl)
    {
        return cef_event_flags_t::EVENTFLAG_CONTROL_DOWN;
    }
    else if (modifiers == Modifiers::kAlt)
    {
        return cef_event_flags_t::EVENTFLAG_ALT_DOWN;
    }
    else if (modifiers == Modifiers::kWin)
    {
        return cef_event_flags_t::EVENTFLAG_COMMAND_DOWN;
    }
    else
    {
        return cef_event_flags_t::EVENTFLAG_NONE;
    }
}

/* =================== IMEControl ================= */

void IMEControl::SetBrowser(CefRefPtr<CefBrowser> browser)
{
    _browser = browser;
}

void IMEControl::OnIMEComposition(std::string input)
{
    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->ImeCommitText(input, CefRange::InvalidRange(), 0);
}

void IMEControl::OnIMESetComposition(std::string input, int x, int y)
{
    if (!_browser.has_value())
    {
        return;
    }

    CefCompositionUnderline line;
    line.style = CEF_CUS_DASH;
    line.range = CefRange(0, y);

    _browser.value()->GetHost()->ImeSetComposition(input, { line }, CefRange::InvalidRange(),
                                                   CefRange(x, y));
}

void IMEControl::IClose()
{
    is_closed = true;
    _browser = std::nullopt;
}

/* =================== IControl ================= */

void IControl::SetBrowser(CefRefPtr<CefBrowser> browser)
{
    _browser = browser;
    IMEControl::SetBrowser(browser);
}

void IControl::OnMouseClick(MouseButtons button, bool pressed)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    if (button == MouseButtons::kLeft)
    {
        _mouse_event.modifiers = EVENTFLAG_LEFT_MOUSE_BUTTON;
    }
    else if (button == MouseButtons::kMiddle)
    {
        _mouse_event.modifiers = EVENTFLAG_MIDDLE_MOUSE_BUTTON;
    }
    else if (button == MouseButtons::kRight)
    {
        _mouse_event.modifiers = EVENTFLAG_RIGHT_MOUSE_BUTTON;
    }

    _browser.value()->GetHost()->SendMouseClickEvent(_mouse_event, from_c(button), !pressed, 1);
}

void IControl::OnMouseClickWithPosition(MouseButtons button, int x, int y, bool pressed)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    if (button == MouseButtons::kLeft)
    {
        _mouse_event.modifiers = EVENTFLAG_LEFT_MOUSE_BUTTON;
    }
    else if (button == MouseButtons::kMiddle)
    {
        _mouse_event.modifiers = EVENTFLAG_MIDDLE_MOUSE_BUTTON;
    }
    else if (button == MouseButtons::kRight)
    {
        _mouse_event.modifiers = EVENTFLAG_RIGHT_MOUSE_BUTTON;
    }

    _mouse_event.x = x;
    _mouse_event.y = y;
    _browser.value()->GetHost()->SendMouseClickEvent(_mouse_event, from_c(button), !pressed, 1);
}

void IControl::OnMouseMove(int x, int y)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _mouse_event.x = x;
    _mouse_event.y = y;
    _browser.value()->GetHost()->SendMouseMoveEvent(_mouse_event, false);
}

void IControl::OnMouseWheel(int x, int y)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    _browser.value()->GetHost()->SendMouseWheelEvent(_mouse_event, x, y);
}

void IControl::OnKeyboard(int scan_code, bool pressed, Modifiers modifiers)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

#ifdef WIN32
    auto windows_key_code = MapVirtualKeyA(scan_code, MAPVK_VSC_TO_VK);
    bool is_capslock_on = (GetKeyState(VK_CAPITAL) & 0x0001) != 0;
#else
    bool is_capslock_on = false;
    if (_display)
    {
        XKeyboardState state;
        XGetKeyboardControl(_display, &state);
        is_capslock_on = state.led_mask & (1 << 1);
        XCloseDisplay(_display);
    }
#endif

    bool is_az = IS_ABC(scan_code);                   // a-z
    bool is_number = IS_NUMBER(scan_code);            // 0-9
    bool is_symbol = is_symbol_from_code(scan_code);  // []\;',./`-=

    CefKeyEvent event;
    event.type = from_c(pressed);
    event.native_key_code = scan_code;
    event.modifiers = from_c(modifiers);
#ifdef WIN32
    event.windows_key_code = windows_key_code;
#endif

    _browser.value()->GetHost()->SendKeyEvent(event);
    if (!pressed)
    {
        return;
    }

    // not allow to char mode
    if (!IS_ALLOW_CHAR_MODE(scan_code))
    {
        return;
    }

    if (!is_capslock_on && is_az && scan_code != ENTER_CODE)
    {
    #ifdef WIN32
        event.windows_key_code += 32;
    #endif
    }

    if (is_number && modifiers == Modifiers::kShift)
    {
        event.windows_key_code = get_symbol_ref(scan_code).value().second;
    }
    else if (is_symbol && modifiers == Modifiers::kShift)
    {
        event.windows_key_code = get_symbol_ref(scan_code).value().second;
    }
    else if (is_symbol)
    {
        event.windows_key_code = get_symbol_ref(scan_code).value().first;
    }

    // allow to char model
    if (is_az || is_number || is_symbol || scan_code == SPACE_CODE)
    {
        event.type = KEYEVENT_CHAR;
        _browser.value()->GetHost()->SendKeyEvent(event);
    }
}

void IControl::OnTouch(int id,
                       int x,
                       int y,
                       cef_touch_event_type_t type,
                       cef_pointer_type_t pointer_type)
{
    if (_is_closed)
    {
        return;
    }

    if (!_browser.has_value())
    {
        return;
    }

    CefTouchEvent event;

    event.id = id;
    event.x = x;
    event.y = y;
    event.type = type;
    event.pointer_type = pointer_type;

    _browser.value()->GetHost()->SendTouchEvent(event);
}

void IControl::IClose()
{
    IMEControl::IClose();

    _is_closed = true;
    _browser = std::nullopt;
}
