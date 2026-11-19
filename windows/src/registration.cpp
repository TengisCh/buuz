// SPDX-License-Identifier: MIT OR Apache-2.0
// DLL entry points, class factory and registration with Windows.
#include <windows.h>

#include <olectl.h>
#include <strsafe.h>

#include <new>
#include <string>

#include "display_attributes.h"
#include "guids.h"
#include "text_service.h"

namespace buuz {

namespace {

LONG g_dll_refs = 0;
HMODULE g_module = nullptr;

constexpr wchar_t kDescription[] = L"Buuz 2.0";

class Factory final : public IClassFactory {
public:
    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override {
        if (!ppv) return E_INVALIDARG;
        if (riid == IID_IUnknown || riid == IID_IClassFactory) {
            *ppv = static_cast<IClassFactory*>(this);
            AddRef();
            return S_OK;
        }
        *ppv = nullptr;
        return E_NOINTERFACE;
    }
    STDMETHODIMP_(ULONG) AddRef() override { return ++refs_; }
    STDMETHODIMP_(ULONG) Release() override {
        const ULONG n = --refs_;
        if (n == 0) delete this;
        return n;
    }
    STDMETHODIMP CreateInstance(IUnknown* outer, REFIID riid, void** ppv) override {
        if (!ppv) return E_INVALIDARG;
        *ppv = nullptr;
        if (outer) return CLASS_E_NOAGGREGATION;
        if (riid == IID_ITfDisplayAttributeProvider) return create_display_attribute_provider(riid, ppv);
        TextService* svc = new (std::nothrow) TextService();
        if (!svc) return E_OUTOFMEMORY;
        const HRESULT hr = svc->QueryInterface(riid, ppv);
        svc->Release();
        return hr;
    }
    STDMETHODIMP LockServer(BOOL lock) override {
        if (lock) {
            dll_add_ref();
        } else {
            dll_release();
        }
        return S_OK;
    }

private:
    ULONG refs_ = 1;
};

std::wstring guid_string(REFGUID g) {
    wchar_t buf[64];
    StringFromGUID2(g, buf, 64);
    return buf;
}

HRESULT set_value(HKEY key, const wchar_t* name, const std::wstring& value) {
    return HRESULT_FROM_WIN32(RegSetValueExW(key, name, 0, REG_SZ,
                                             reinterpret_cast<const BYTE*>(value.c_str()),
                                             static_cast<DWORD>((value.size() + 1) * sizeof(wchar_t))));
}

}  // namespace

void dll_add_ref() { InterlockedIncrement(&g_dll_refs); }
void dll_release() { InterlockedDecrement(&g_dll_refs); }
bool dll_can_unload() { return g_dll_refs == 0; }

HRESULT create_class_factory(REFIID riid, void** ppv) {
    Factory* f = new (std::nothrow) Factory();
    if (!f) return E_OUTOFMEMORY;
    const HRESULT hr = f->QueryInterface(riid, ppv);
    f->Release();
    return hr;
}

HRESULT register_server(HMODULE module) {
    wchar_t path[MAX_PATH];
    if (GetModuleFileNameW(module, path, MAX_PATH) == 0) return HRESULT_FROM_WIN32(GetLastError());

    // 1. COM class: HKLM\SOFTWARE\Classes\CLSID\{...}\InprocServer32
    const std::wstring clsid = guid_string(CLSID_BuuzTextService);
    const std::wstring base = L"SOFTWARE\\Classes\\CLSID\\" + clsid;
    HKEY key = nullptr;
    LONG rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE, base.c_str(), 0, nullptr, 0,
                              KEY_WRITE, nullptr, &key, nullptr);
    if (rc != ERROR_SUCCESS) return HRESULT_FROM_WIN32(rc);
    set_value(key, nullptr, kDescription);
    RegCloseKey(key);
    rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE, (base + L"\\InprocServer32").c_str(), 0, nullptr, 0,
                         KEY_WRITE, nullptr, &key, nullptr);
    if (rc != ERROR_SUCCESS) return HRESULT_FROM_WIN32(rc);
    set_value(key, nullptr, path);
    set_value(key, L"ThreadingModel", L"Apartment");
    RegCloseKey(key);

    // 2. Language profile: Mongolian with Buuz, US keys underneath.
    ITfInputProcessorProfileMgr* mgr = nullptr;
    HRESULT hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER,
                                  IID_ITfInputProcessorProfileMgr, reinterpret_cast<void**>(&mgr));
    if (FAILED(hr)) return hr;
    const HKL us_layout = reinterpret_cast<HKL>(static_cast<ULONG_PTR>(0x04090409));
    hr = mgr->RegisterProfile(CLSID_BuuzTextService, kLangMongolian, GUID_BuuzProfile, kDescription,
                              static_cast<ULONG>(wcslen(kDescription)), path,
                              static_cast<ULONG>(wcslen(path)), 0, us_layout, 0, TRUE, 0);
    mgr->Release();
    if (FAILED(hr)) return hr;

    // 3. Categories: a keyboard TIP, with display attributes, for Store apps
    //    (immersive mode) and the system tray.
    ITfCategoryMgr* cat = nullptr;
    hr = CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER, IID_ITfCategoryMgr,
                          reinterpret_cast<void**>(&cat));
    if (FAILED(hr)) return hr;
    const GUID* cats[] = {&GUID_TFCAT_TIP_KEYBOARD, &GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
                          &GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT, &GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
                          &GUID_TFCAT_TIPCAP_UIELEMENTENABLED};
    for (const GUID* c : cats) {
        hr = cat->RegisterCategory(CLSID_BuuzTextService, *c, CLSID_BuuzTextService);
        if (FAILED(hr)) break;
    }
    cat->Release();
    return hr;
}

HRESULT unregister_server() {
    ITfInputProcessorProfileMgr* mgr = nullptr;
    if (SUCCEEDED(CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER,
                                   IID_ITfInputProcessorProfileMgr, reinterpret_cast<void**>(&mgr)))) {
        mgr->UnregisterProfile(CLSID_BuuzTextService, kLangMongolian, GUID_BuuzProfile, 0);
        mgr->Release();
    }
    ITfCategoryMgr* cat = nullptr;
    if (SUCCEEDED(CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER,
                                   IID_ITfCategoryMgr, reinterpret_cast<void**>(&cat)))) {
        const GUID* cats[] = {&GUID_TFCAT_TIP_KEYBOARD, &GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
                              &GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT, &GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
                              &GUID_TFCAT_TIPCAP_UIELEMENTENABLED};
        for (const GUID* c : cats) cat->UnregisterCategory(CLSID_BuuzTextService, *c, CLSID_BuuzTextService);
        cat->Release();
    }
    const std::wstring base = L"SOFTWARE\\Classes\\CLSID\\" + guid_string(CLSID_BuuzTextService);
    RegDeleteTreeW(HKEY_LOCAL_MACHINE, base.c_str());
    return S_OK;
}

}  // namespace buuz

BOOL APIENTRY DllMain(HMODULE module, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        buuz::g_module = module;
        DisableThreadLibraryCalls(module);
    }
    return TRUE;
}

STDAPI DllGetClassObject(REFCLSID clsid, REFIID riid, void** ppv) {
    if (!ppv) return E_INVALIDARG;
    *ppv = nullptr;
    if (clsid != CLSID_BuuzTextService) return CLASS_E_CLASSNOTAVAILABLE;
    return buuz::create_class_factory(riid, ppv);
}

STDAPI DllCanUnloadNow() { return buuz::dll_can_unload() ? S_OK : S_FALSE; }

STDAPI DllRegisterServer() {
    const HRESULT co = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    const HRESULT hr = buuz::register_server(buuz::g_module);
    if (SUCCEEDED(co)) CoUninitialize();
    if (FAILED(hr)) buuz::unregister_server();
    return hr;
}

STDAPI DllUnregisterServer() {
    const HRESULT co = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    const HRESULT hr = buuz::unregister_server();
    if (SUCCEEDED(co)) CoUninitialize();
    return hr;
}
