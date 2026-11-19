// SPDX-License-Identifier: MIT OR Apache-2.0
// Buuz settings and onboarding app. Plain Win32, no extra runtime. It runs
// apart from the text service DLL, so it never touches the typing path.
#include <windows.h>
#include <msctf.h>
#include <shellapi.h>
#include <shlobj.h>

#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "app_state.h"
#include "guids.h"
#include "strings.h"

using namespace buuz;

namespace {

constexpr int kWidth = 760;
constexpr int kHeight = 540;
constexpr UINT_PTR kTimerAdded = 1;
constexpr int kIdLang = 900;
constexpr int kIdPrimary = 901;
constexpr int kIdBack = 902;
constexpr int kIdSecondary = 903;
constexpr int kIdNavBase = 920;
constexpr int kIdOptionBase = 940;
constexpr int kIdTheme = 960;
constexpr int kIdRerun = 961;
constexpr int kIdFeedback = 962;
constexpr int kIdTryEdit = 970;

constexpr COLORREF kBlue = RGB(0x2F, 0x6F, 0xDE);  // Khadag Blue, primary action

enum Step { kWelcome, kPrivacy, kAdd, kShortcut, kTry, kTip, kDone };
enum Page { kHome, kTyping, kShortcuts, kPrivacyPage, kGeneral, kAbout, kPageCount };

struct App {
    HWND hwnd = nullptr;
    HFONT font = nullptr, font_big = nullptr, font_title = nullptr;
    HBRUSH bg = nullptr;
    AppState state;
    std::string conf;  // settings.conf text
    bool onboarding = true;
    int page = kHome;
    std::vector<HWND> controls;
    HWND add_status = nullptr, shortcut_status = nullptr, try_edit = nullptr;
    HWND tasks[3] = {};
};

App g;

// --- files ---------------------------------------------------------------

std::wstring data_dir() {
    wchar_t buf[MAX_PATH] = {};
    if (FAILED(SHGetFolderPathW(nullptr, CSIDL_APPDATA, nullptr, 0, buf))) return std::wstring();
    std::wstring dir = std::wstring(buf) + L"\\Buuz";
    CreateDirectoryW(dir.c_str(), nullptr);
    return dir;
}

std::string read_file(const std::wstring& path) {
    std::ifstream f(path.c_str(), std::ios::binary);
    std::ostringstream ss;
    ss << f.rdbuf();
    return ss.str();
}

void write_file(const std::wstring& path, const std::string& text) {
    std::ofstream f(path.c_str(), std::ios::binary | std::ios::trunc);
    f << text;
}

void save_state() { write_file(data_dir() + L"\\app.conf", write_app_state(read_file(data_dir() + L"\\app.conf"), g.state)); }
void save_conf() { write_file(data_dir() + L"\\settings.conf", g.conf); }

// --- text ----------------------------------------------------------------

std::wstring widen(const std::string& u8) {
    if (u8.empty()) return std::wstring();
    int n = MultiByteToWideChar(CP_UTF8, 0, u8.data(), static_cast<int>(u8.size()), nullptr, 0);
    std::wstring w(static_cast<size_t>(n), L'\0');
    MultiByteToWideChar(CP_UTF8, 0, u8.data(), static_cast<int>(u8.size()), &w[0], n);
    return w;
}

std::wstring T(const char* id) { return widen(tr(g.state.lang, id)); }

// --- Buuz profile --------------------------------------------------------

bool buuz_added() {
    ITfInputProcessorProfiles* profiles = nullptr;
    if (FAILED(CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER,
                                IID_ITfInputProcessorProfiles, reinterpret_cast<void**>(&profiles))))
        return false;
    BOOL enabled = FALSE;
    HRESULT hr = profiles->IsEnabledLanguageProfile(CLSID_BuuzTextService, kLangMongolian, GUID_BuuzProfile, &enabled);
    profiles->Release();
    return SUCCEEDED(hr) && enabled;
}

std::wstring guid_text(const GUID& g_) {
    wchar_t buf[64] = {};
    StringFromGUID2(g_, buf, 64);
    return buf;
}

// Add Mongolian with Buuz to the user's keyboard list.
void add_buuz() {
    using Fn = BOOL(WINAPI*)(LPCWSTR, DWORD);
    HMODULE input = LoadLibraryW(L"input.dll");
    if (!input) return;
    Fn install = reinterpret_cast<Fn>(reinterpret_cast<void*>(GetProcAddress(input, "InstallLayoutOrTip")));
    if (install) install((L"0450:" + guid_text(CLSID_BuuzTextService) + guid_text(GUID_BuuzProfile)).c_str(), 0);
    FreeLibrary(input);
}

// --- controls ------------------------------------------------------------

HWND add(HWND h) {
    g.controls.push_back(h);
    return h;
}

HWND label(const std::wstring& text, int x, int y, int w, int h, HFONT font = nullptr, DWORD style = 0) {
    HWND c = CreateWindowExW(0, L"STATIC", text.c_str(), WS_CHILD | WS_VISIBLE | style, x, y, w, h, g.hwnd, nullptr,
                             nullptr, nullptr);
    SendMessageW(c, WM_SETFONT, reinterpret_cast<WPARAM>(font ? font : g.font), TRUE);
    return add(c);
}

HWND button(const std::wstring& text, int id, int x, int y, int w, int h, DWORD style = BS_PUSHBUTTON) {
    HWND c = CreateWindowExW(0, L"BUTTON", text.c_str(), WS_CHILD | WS_VISIBLE | WS_TABSTOP | style, x, y, w, h, g.hwnd,
                             reinterpret_cast<HMENU>(static_cast<INT_PTR>(id)), nullptr, nullptr);
    SendMessageW(c, WM_SETFONT, reinterpret_cast<WPARAM>(g.font), TRUE);
    return add(c);
}

HWND primary(const std::wstring& text, int x, int y) { return button(text, kIdPrimary, x, y, 170, 40, BS_OWNERDRAW); }

void clear_controls() {
    KillTimer(g.hwnd, kTimerAdded);
    for (HWND c : g.controls) DestroyWindow(c);
    g.controls.clear();
    g.add_status = g.shortcut_status = g.try_edit = nullptr;
    for (HWND& t : g.tasks) t = nullptr;
}

void build();

void go_step(int step) {
    g.state.step = step;
    save_state();
    build();
}

void finish_onboarding() {
    g.state.onboarded = true;
    g.state.step = 0;
    save_state();
    g.onboarding = false;
    g.page = kHome;
    build();
}

// --- pages ---------------------------------------------------------------

void common_nav(bool can_back, const std::wstring& next_text, bool next_enabled = true) {
    if (can_back) button(T("btn_back"), kIdBack, 40, 460, 100, 36);
    HWND p = primary(next_text, kWidth - 40 - 170 - 16, 454);
    EnableWindow(p, next_enabled);
}

void update_add_status() {
    if (!g.add_status) return;
    bool ok = buuz_added();
    SetWindowTextW(g.add_status, (ok ? L"✅  " + T("add_status_done") : L"⏳  " + T("add_status_waiting")).c_str());
    HWND next = GetDlgItem(g.hwnd, kIdPrimary);
    if (next) EnableWindow(next, ok);
}

void update_try_tasks() {
    if (!g.try_edit) return;
    wchar_t buf[512] = {};
    GetWindowTextW(g.try_edit, buf, 512);
    const std::wstring text = buf;
    const char* ids[3] = {"try_task1", "try_task2", "try_task3"};
    const wchar_t* expect[3] = {L"сайн байна уу", L"баярлалаа", L"өнөөдөр"};
    for (int i = 0; i < 3; ++i) {
        bool done = text.find(expect[i]) != std::wstring::npos;
        SetWindowTextW(g.tasks[i], ((done ? L"✅  " : L"○  ") + T(ids[i])).c_str());
    }
}

void page_welcome() {
    label(L"🥟", 40, 40, 100, 90, g.font_title);
    label(T("welcome_title"), 40, 140, 680, 50, g.font_big);
    label(T("welcome_line"), 40, 200, 680, 30);
    label(widen(tr(g.state.lang, "welcome_demo_in")) + L"   →   " + widen(tr(g.state.lang, "welcome_demo_out")), 40, 250,
          680, 40, g.font_big);
    label(T("welcome_credit"), 40, 400, 680, 40);
    primary(T("btn_start"), kWidth - 40 - 170 - 16, 454);
}

void page_privacy() {
    label(T("privacy_title"), 40, 70, 680, 50, g.font_big);
    const char* ids[4] = {"privacy_offline", "privacy_nokeys", "privacy_noaccount", "privacy_password"};
    const wchar_t* icons[4] = {L"📴", L"🙈", L"👤", L"🔐"};
    for (int i = 0; i < 4; ++i) label(std::wstring(icons[i]) + L"   " + T(ids[i]), 60, 150 + i * 60, 640, 40, g.font_big);
    common_nav(true, T("btn_gotit"));
}

void page_add() {
    label(T("add_title"), 40, 70, 680, 50, g.font_big);
    label(T("add_line"), 40, 130, 680, 30);
    g.add_status = label(L"", 40, 190, 400, 40, g.font_big);
    button(T("btn_add"), kIdSecondary, 40, 250, 170, 40);
    label(T("add_manual"), 40, 330, 680, 60);
    button(T("btn_open_settings"), kIdSecondary + 1, 40, 400, 240, 36);
    common_nav(true, T("btn_next"), false);
    update_add_status();
    SetTimer(g.hwnd, kTimerAdded, 1000, nullptr);
}

void page_shortcut() {
    label(T("shortcut_title"), 40, 70, 680, 50, g.font_big);
    label(L"⭐  ⊞  " + T("shortcut_win_space"), 60, 150, 640, 36, g.font_big);
    label(L"⌨️  " + T("shortcut_alt_shift"), 60, 200, 640, 36, g.font_big);
    HWND soon = label(L"⌃  " + T("shortcut_ctrl_space") + L"  (" + T("typing_soon") + L")", 60, 250, 640, 36, g.font_big);
    EnableWindow(soon, FALSE);
    label(T("shortcut_try") + L" 👉", 40, 330, 680, 30);
    g.shortcut_status = label(L"", 40, 370, 680, 40, g.font_big);
    common_nav(true, T("btn_next"));
}

void page_try() {
    label(T("try_title"), 40, 70, 680, 50, g.font_big);
    label(T("try_hint"), 40, 125, 680, 30);
    const char* ids[3] = {"try_task1", "try_task2", "try_task3"};
    for (int i = 0; i < 3; ++i) g.tasks[i] = label(L"", 40, 170 + i * 34, 680, 30);
    g.try_edit = CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", L"", WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_AUTOHSCROLL, 40,
                                 290, 680, 40, g.hwnd, reinterpret_cast<HMENU>(static_cast<INT_PTR>(kIdTryEdit)), nullptr,
                                 nullptr);
    SendMessageW(g.try_edit, WM_SETFONT, reinterpret_cast<WPARAM>(g.font_big), TRUE);
    SendMessageW(g.try_edit, EM_SETCUEBANNER, TRUE, reinterpret_cast<LPARAM>(T("try_placeholder").c_str()));
    add(g.try_edit);
    (void)ids;
    update_try_tasks();
    common_nav(true, T("btn_next"));
    button(T("btn_skip"), kIdSecondary, kWidth - 40 - 170 - 16 - 110, 458, 100, 32);
}

void page_tip() {
    label(T("tip_title"), 40, 70, 680, 50, g.font_big);
    label(L"🪄  " + T("tip_harmony_title"), 40, 150, 680, 36, g.font_big);
    label(T("tip_harmony_line"), 80, 190, 640, 30);
    label(L"🎯  " + T("tip_keys_title"), 40, 260, 680, 36, g.font_big);
    label(T("tip_keys_line"), 80, 300, 640, 30);
    common_nav(true, T("btn_next"));
}

void page_done() {
    label(L"🎉", 40, 50, 100, 90, g.font_title);
    label(T("done_title"), 40, 150, 680, 50, g.font_big);
    label(T("done_where"), 40, 220, 680, 30);
    label(T("done_recap"), 40, 260, 680, 30);
    label(T("home_cheat"), 40, 300, 680, 30);
    primary(T("btn_finish"), kWidth - 40 - 170 - 16, 454);
}

void page_settings() {
    // Sidebar.
    const char* nav[kPageCount] = {"nav_home", "nav_typing", "nav_shortcuts", "nav_privacy", "nav_general", "nav_about"};
    const wchar_t* icons[kPageCount] = {L"🏠", L"✍️", L"⌨️", L"🔒", L"⚙️", L"ℹ️"};
    for (int i = 0; i < kPageCount; ++i) {
        HWND b = button(std::wstring(icons[i]) + L"  " + T(nav[i]), kIdNavBase + i, 16, 70 + i * 48, 180, 40,
                        BS_PUSHBUTTON | BS_LEFT);
        if (i == g.page) EnableWindow(b, FALSE);
    }
    const int x = 230, w = 490;
    switch (g.page) {
        case kHome: {
            bool ok = buuz_added();
            label((ok ? L"✅  " + T("home_ok") : L"⚠️  " + T("home_fix")), x, 80, w, 40, g.font_big);
            if (!ok) button(T("btn_add"), kIdSecondary, x, 140, 170, 40);
            label(T("home_cheat"), x, 220, w, 30);
            break;
        }
        case kTyping: {
            size_t n = 0;
            const TypingOption* opts = typing_options(&n);
            for (size_t i = 0; i < n; ++i) {
                std::wstring text = T(opts[i].label_id);
                if (opts[i].soon) text += L"  (" + T("typing_soon") + L")";
                HWND c = button(text, kIdOptionBase + static_cast<int>(i), x, 80 + static_cast<int>(i) * 44, w, 36,
                                BS_AUTOCHECKBOX);
                SendMessageW(c, BM_SETCHECK, option_on(g.conf, opts[i]) ? BST_CHECKED : BST_UNCHECKED, 0);
                if (opts[i].soon) EnableWindow(c, FALSE);
            }
            break;
        }
        case kShortcuts:
            label(L"⭐  ⊞  " + T("shortcut_win_space"), x, 80, w, 36, g.font_big);
            label(L"⌨️  " + T("shortcut_alt_shift"), x, 130, w, 36, g.font_big);
            break;
        case kPrivacyPage: {
            const char* ids[4] = {"privacy_offline", "privacy_nokeys", "privacy_noaccount", "privacy_password"};
            const wchar_t* ic[4] = {L"📴", L"🙈", L"👤", L"🔐"};
            for (int i = 0; i < 4; ++i) label(std::wstring(ic[i]) + L"   " + T(ids[i]), x, 80 + i * 50, w, 36, g.font_big);
            break;
        }
        case kGeneral:
            label(T("general_language"), x, 80, w, 30);
            button(g.state.lang == Lang::Mongolian ? L"Монгол → English" : L"English → Монгол", kIdLang, x, 112, 200, 36);
            label(T("general_theme"), x, 180, w, 30);
            button(g.state.dark ? T("general_theme_dark") : T("general_theme_light"), kIdTheme, x, 212, 200, 36);
            button(T("general_rerun"), kIdRerun, x, 290, 260, 36);
            break;
        case kAbout:
            label(L"🥟  Buuz 2.0", x, 80, w, 40, g.font_big);
            label(T("about_credit"), x, 140, w, 50);
            label(T("about_license"), x, 200, w, 30);
            button(L"💬  " + T("about_feedback"), kIdFeedback, x, 250, 240, 36);
            break;
    }
}

void build() {
    clear_controls();
    // Language switch: shows the other language, as on the website.
    button(widen(tr(g.state.lang, "lang_toggle")), kIdLang, kWidth - 40 - 60, 12, 60, 30);
    if (!g.onboarding) {
        page_settings();
    } else {
        switch (g.state.step) {
            case kWelcome: page_welcome(); break;
            case kPrivacy: page_privacy(); break;
            case kAdd: page_add(); break;
            case kShortcut: page_shortcut(); break;
            case kTry: page_try(); break;
            case kTip: page_tip(); break;
            default: page_done(); break;
        }
    }
    InvalidateRect(g.hwnd, nullptr, TRUE);
}

void toggle_lang() {
    g.state.lang = other(g.state.lang);
    save_state();
    build();
}

void on_command(int id, int code) {
    if (id == kIdLang) return toggle_lang();
    if (id == kIdTryEdit) {
        if (code == EN_CHANGE) update_try_tasks();
        return;
    }
    if (g.onboarding) {
        if (id == kIdPrimary) return g.state.step >= kDone ? finish_onboarding() : go_step(g.state.step + 1);
        if (id == kIdBack && g.state.step > 0) return go_step(g.state.step - 1);
        if (id == kIdSecondary) {
            if (g.state.step == kAdd) return add_buuz();
            if (g.state.step == kTry) return go_step(g.state.step + 1);
        }
        if (id == kIdSecondary + 1) ShellExecuteW(nullptr, L"open", L"ms-settings:regionlanguage", nullptr, nullptr, SW_SHOW);
        return;
    }
    if (id >= kIdNavBase && id < kIdNavBase + kPageCount) {
        g.page = id - kIdNavBase;
        return build();
    }
    if (id >= kIdOptionBase && id < kIdOptionBase + 20) {
        size_t n = 0;
        const TypingOption* opts = typing_options(&n);
        size_t i = static_cast<size_t>(id - kIdOptionBase);
        if (i < n && !opts[i].soon) {
            bool on = SendMessageW(GetDlgItem(g.hwnd, id), BM_GETCHECK, 0, 0) == BST_CHECKED;
            g.conf = set_option(g.conf, opts[i], on);
            save_conf();
        }
        return;
    }
    if (id == kIdSecondary) {
        add_buuz();
        return build();
    }
    if (id == kIdTheme) {
        g.state.dark = !g.state.dark;
        save_state();
        DeleteObject(g.bg);
        g.bg = CreateSolidBrush(g.state.dark ? RGB(0x1E, 0x1E, 0x24) : RGB(0xFF, 0xFF, 0xFF));
        return build();
    }
    if (id == kIdRerun) {
        g.onboarding = true;
        g.state.step = 0;
        return build();
    }
    if (id == kIdFeedback) ShellExecuteW(nullptr, L"open", L"https://github.com/TengisCh/buuz/issues", nullptr, nullptr, SW_SHOW);
}

void draw_primary(const DRAWITEMSTRUCT* d) {
    HBRUSH b = CreateSolidBrush(kBlue);
    HBRUSH edge = CreateSolidBrush(RGB(0x1F, 0x4F, 0xA8));  // the pressed 4 px bottom edge
    RECT r = d->rcItem;
    const bool disabled = (d->itemState & ODS_DISABLED) != 0;
    if (disabled) {
        DeleteObject(b);
        b = CreateSolidBrush(RGB(0xB8, 0xC4, 0xDC));
    }
    FillRect(d->hDC, &r, b);
    RECT bottom = r;
    bottom.top = r.bottom - 4;
    FillRect(d->hDC, &bottom, disabled ? b : edge);
    wchar_t text[128] = {};
    GetWindowTextW(d->hwndItem, text, 128);
    SetBkMode(d->hDC, TRANSPARENT);
    SetTextColor(d->hDC, RGB(255, 255, 255));
    HGDIOBJ old = SelectObject(d->hDC, g.font);
    r.bottom -= 2;
    DrawTextW(d->hDC, text, -1, &r, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
    SelectObject(d->hDC, old);
    if (d->itemState & ODS_FOCUS) {
        RECT f = d->rcItem;
        InflateRect(&f, -3, -3);
        DrawFocusRect(d->hDC, &f);
    }
    DeleteObject(b);
    DeleteObject(edge);
}

LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    switch (msg) {
        case WM_COMMAND: on_command(LOWORD(wp), HIWORD(wp)); return 0;
        case WM_TIMER:
            if (wp == kTimerAdded) update_add_status();
            return 0;
        case WM_INPUTLANGCHANGE:
            if (g.shortcut_status) SetWindowTextW(g.shortcut_status, (L"✅  " + T("shortcut_ok")).c_str());
            return 0;
        case WM_DRAWITEM:
            if (wp == kIdPrimary) {
                draw_primary(reinterpret_cast<const DRAWITEMSTRUCT*>(lp));
                return TRUE;
            }
            break;
        case WM_CTLCOLORSTATIC:
        case WM_CTLCOLORBTN: {
            HDC dc = reinterpret_cast<HDC>(wp);
            SetBkColor(dc, g.state.dark ? RGB(0x1E, 0x1E, 0x24) : RGB(0xFF, 0xFF, 0xFF));
            SetTextColor(dc, g.state.dark ? RGB(0xEE, 0xEE, 0xF2) : RGB(0x22, 0x22, 0x2A));
            return reinterpret_cast<LRESULT>(g.bg);
        }
        case WM_ERASEBKGND: {
            RECT r;
            GetClientRect(hwnd, &r);
            FillRect(reinterpret_cast<HDC>(wp), &r, g.bg);
            return 1;
        }
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;
        default: break;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

HFONT make_font(int pt, int weight) {
    HDC dc = GetDC(nullptr);
    int h = -MulDiv(pt, GetDeviceCaps(dc, LOGPIXELSY), 72);
    ReleaseDC(nullptr, dc);
    return CreateFontW(h, 0, 0, 0, weight, FALSE, FALSE, FALSE, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                       CLEARTYPE_QUALITY, DEFAULT_PITCH, L"Segoe UI");
}

}  // namespace

int WINAPI wWinMain(HINSTANCE inst, HINSTANCE, PWSTR, int show) {
    // One window only: a second start brings the first to the front.
    HANDLE once = CreateMutexW(nullptr, TRUE, L"Local\\BuuzSettingsApp");
    if (GetLastError() == ERROR_ALREADY_EXISTS) {
        HWND other_window = FindWindowW(L"BuuzSettingsWindow", nullptr);
        if (other_window) SetForegroundWindow(other_window);
        return 0;
    }
    CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);

    g.state = parse_app_state(read_file(data_dir() + L"\\app.conf"));
    g.conf = read_file(data_dir() + L"\\settings.conf");
    g.onboarding = !g.state.onboarded;
    g.bg = CreateSolidBrush(g.state.dark ? RGB(0x1E, 0x1E, 0x24) : RGB(0xFF, 0xFF, 0xFF));
    g.font = make_font(11, FW_NORMAL);
    g.font_big = make_font(16, FW_SEMIBOLD);
    g.font_title = make_font(40, FW_NORMAL);

    WNDCLASSW wc = {};
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = inst;
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.lpszClassName = L"BuuzSettingsWindow";
    RegisterClassW(&wc);

    RECT r = {0, 0, kWidth, kHeight};
    const DWORD style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;  // fixed size
    AdjustWindowRect(&r, style, FALSE);
    int w = r.right - r.left, h = r.bottom - r.top;
    int x = (GetSystemMetrics(SM_CXSCREEN) - w) / 2, y = (GetSystemMetrics(SM_CYSCREEN) - h) / 2;
    g.hwnd = CreateWindowExW(0, wc.lpszClassName, L"Buuz 2.0", style, x, y, w, h, nullptr, nullptr, inst, nullptr);
    build();
    ShowWindow(g.hwnd, show);

    MSG m;
    while (GetMessageW(&m, nullptr, 0, 0) > 0) {
        if (!IsDialogMessageW(g.hwnd, &m)) {
            TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }
    CoUninitialize();
    CloseHandle(once);
    return 0;
}
