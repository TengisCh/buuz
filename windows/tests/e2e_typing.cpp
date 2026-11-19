// SPDX-License-Identifier: MIT OR Apache-2.0
// End-to-end test, Windows only. It makes a real window with a Rich Edit
// control, turns the Buuz text service on for this thread, sends real key
// events with SendInput, and reads back the text that the control holds.
// Run it after `regsvr32 buuz_tsf.dll`. Exit code 0 means the text service
// turned `sain baina` into `сайн байна` inside a real app.
#include <windows.h>

#include <msctf.h>
#include <richedit.h>

#include <cstdio>
#include <string>

#include "../src/guids.h"

namespace {

HWND g_edit = nullptr;

LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_SETFOCUS && g_edit) {
        SetFocus(g_edit);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

void pump(DWORD ms) {
    const DWORD end = GetTickCount() + ms;
    do {
        MSG msg;
        while (PeekMessageW(&msg, nullptr, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(5);
    } while (static_cast<LONG>(end - GetTickCount()) > 0);
}

UINT g_sent = 0;
UINT g_sent_expected = 0;

void send_key(WORD vk) {
    INPUT in[2] = {};
    in[0].type = INPUT_KEYBOARD;
    in[0].ki.wVk = vk;
    in[0].ki.wScan = static_cast<WORD>(MapVirtualKeyW(vk, MAPVK_VK_TO_VSC));
    in[1] = in[0];
    in[1].ki.dwFlags = KEYEVENTF_KEYUP;
    g_sent += SendInput(2, in, sizeof(INPUT));
    g_sent_expected += 2;
    pump(60);
}

void type(const char* keys) {
    for (const char* p = keys; *p; ++p) {
        const char c = *p;
        send_key(c == ' ' ? VK_SPACE : static_cast<WORD>(c >= 'a' ? c - 32 : c));
    }
}

// Bring the window to the front and put the keyboard focus in the control.
// Windows refuses SetForegroundWindow from a background process. Sharing
// the input queue of the current foreground thread makes it work.
void take_focus(HWND top) {
    // Allow this process to take the foreground at once (no lock timeout).
    SystemParametersInfoW(SPI_SETFOREGROUNDLOCKTIMEOUT, 0, nullptr, SPIF_SENDCHANGE);
    AllowSetForegroundWindow(ASFW_ANY);
    const DWORD me = GetCurrentThreadId();
    const HWND fg = GetForegroundWindow();
    const DWORD fg_thread = fg ? GetWindowThreadProcessId(fg, nullptr) : 0;
    const bool attached = fg_thread != 0 && fg_thread != me && AttachThreadInput(me, fg_thread, TRUE);
    ShowWindow(top, SW_SHOW);
    BringWindowToTop(top);
    SwitchToThisWindow(top, TRUE);
    SetForegroundWindow(top);
    SetFocus(g_edit);
    if (attached) AttachThreadInput(me, fg_thread, FALSE);
    pump(300);
}

std::wstring read_text() {
    const int n = GetWindowTextLengthW(g_edit);
    std::wstring s(static_cast<size_t>(n) + 1, L'\0');
    GetWindowTextW(g_edit, &s[0], n + 1);
    s.resize(static_cast<size_t>(n));
    return s;
}

void print_hex(const wchar_t* label, const std::wstring& s) {
    std::printf("%ls (%zu units):", label, s.size());
    for (wchar_t c : s) std::printf(" U+%04X", static_cast<unsigned>(c));
    std::printf("\n");
}

}  // namespace

// If anything blocks, end the test instead of hanging the CI job.
DWORD WINAPI watchdog(LPVOID) {
    Sleep(90000);
    std::printf("FAIL: the test took more than 90 seconds\n");
    std::fflush(stdout);
    ExitProcess(3);
}

// True when mn.txt is in the folder of the loaded buuz_tsf.dll.
bool word_list_next_to_dll() {
    HMODULE dll = GetModuleHandleW(L"buuz_tsf.dll");
    if (!dll) return false;
    wchar_t path[MAX_PATH];
    const DWORD n = GetModuleFileNameW(dll, path, MAX_PATH);
    if (n == 0 || n >= MAX_PATH) return false;
    std::wstring file(path, n);
    const size_t slash = file.find_last_of(L'\\');
    if (slash == std::wstring::npos) return false;
    file.resize(slash + 1);
    file += L"mn.txt";
    return GetFileAttributesW(file.c_str()) != INVALID_FILE_ATTRIBUTES;
}

int main() {
    CreateThread(nullptr, 0, watchdog, nullptr, 0, nullptr);
    if (FAILED(CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED))) return 2;
    LoadLibraryW(L"Msftedit.dll");

    WNDCLASSW wc = {};
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = GetModuleHandleW(nullptr);
    wc.lpszClassName = L"BuuzE2E";
    RegisterClassW(&wc);
    HWND top = CreateWindowExW(0, L"BuuzE2E", L"Buuz end-to-end test", WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                               50, 50, 600, 300, nullptr, nullptr, wc.hInstance, nullptr);
    g_edit = CreateWindowExW(0, MSFTEDIT_CLASS, L"", WS_CHILD | WS_VISIBLE | ES_MULTILINE, 0, 0, 580, 250,
                             top, nullptr, wc.hInstance, nullptr);
    if (!top || !g_edit) {
        std::printf("FAIL: could not create the window\n");
        return 2;
    }
    SendMessageW(g_edit, EM_SETEDITSTYLE, SES_USECTF, SES_USECTF);
    ShowWindow(top, SW_SHOW);
    SetForegroundWindow(top);
    SetFocus(g_edit);
    pump(500);

    ITfInputProcessorProfileMgr* mgr = nullptr;
    HRESULT hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER,
                                  IID_ITfInputProcessorProfileMgr, reinterpret_cast<void**>(&mgr));
    if (FAILED(hr)) {
        std::printf("FAIL: no profile manager (0x%08lX)\n", static_cast<unsigned long>(hr));
        return 2;
    }
    hr = mgr->ActivateProfile(TF_PROFILETYPE_INPUTPROCESSOR, kLangMongolian, CLSID_BuuzTextService,
                              GUID_BuuzProfile, nullptr,
                              TF_IPPMF_FORPROCESS | TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE);
    mgr->Release();
    std::printf("ActivateProfile: 0x%08lX\n", static_cast<unsigned long>(hr));
    pump(1500);
    take_focus(top);

    std::printf("foreground is the test window: %s, focus is the control: %s\n",
                GetForegroundWindow() == top ? "yes" : "no", GetFocus() == g_edit ? "yes" : "no");
    {
        wchar_t cls[128] = L"(none)";
        wchar_t title[128] = L"";
        const HWND fg = GetForegroundWindow();
        if (fg) {
            GetClassNameW(fg, cls, 128);
            GetWindowTextW(fg, title, 128);
        }
        std::printf("foreground window: %p class=%ls title=%ls\n", static_cast<void*>(fg), cls, title);
    }
    const bool loaded = GetModuleHandleW(L"buuz_tsf.dll") != nullptr;
    std::printf("buuz_tsf.dll loaded into this process: %s\n", loaded ? "yes" : "no");

    type("sain baina");
    pump(300);
    std::printf("SendInput accepted %u of %u events\n", g_sent, g_sent_expected);

    std::wstring got = read_text();
    if (got.empty()) {
        // The keys may have gone elsewhere. Take the focus again and retry once.
        std::printf("no text yet: trying again\n");
        take_focus(top);
        type("sain baina");
        pump(500);
        std::printf("SendInput accepted %u of %u events\n", g_sent, g_sent_expected);
        got = read_text();
    }
    const std::wstring want = L"сайн байна";  // сайн байна
    print_hex(L"got ", got);
    print_hex(L"want", want);
    if (got == want) {
        std::printf("PASS: sain baina -> сайн байна in a real Rich Edit control\n");
        // Word choice needs mn.txt next to the DLL (#66: ий over ы).
        if (!word_list_next_to_dll()) {
            std::printf("SKIP: no mn.txt next to buuz_tsf.dll, word choice not tested\n");
            return 0;
        }
        type(" surguuliin ");
        pump(500);
        got = read_text();
        const std::wstring want2 = want + L" сургуулийн ";  // сургуулийн
        print_hex(L"got ", got);
        print_hex(L"want", want2);
        if (got == want2) {
            std::printf("PASS: surguuliin -> сургуулийн with the word list\n");
            return 0;
        }
        std::printf("FAIL: the word list did not reach the text service\n");
        return 1;
    }
    std::printf("FAIL: the text service did not convert the keys\n");
    return 1;
}
