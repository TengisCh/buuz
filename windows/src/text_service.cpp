// SPDX-License-Identifier: MIT OR Apache-2.0
#include "text_service.h"

#include <windows.h>

#include <functional>
#include <mutex>
#include <new>
#include <string>

#include "edit_plan.h"
#include "guids.h"

namespace buuz {

namespace {

// Runs a function inside an edit session.
class EditSession final : public ITfEditSession {
public:
    using Fn = std::function<HRESULT(TfEditCookie)>;
    explicit EditSession(Fn fn) : fn_(std::move(fn)) {}
    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override {
        if (!ppv) return E_INVALIDARG;
        if (riid == IID_IUnknown || riid == IID_ITfEditSession) {
            *ppv = static_cast<ITfEditSession*>(this);
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
    STDMETHODIMP DoEditSession(TfEditCookie ec) override { return fn_(ec); }

private:
    Fn fn_;
    ULONG refs_ = 1;
};

HRESULT run_session(ITfContext* ctx, TfClientId id, EditSession::Fn fn) {
    EditSession* s = new (std::nothrow) EditSession(std::move(fn));
    if (!s) return E_OUTOFMEMORY;
    HRESULT session_hr = E_FAIL;
    // The key handler may ask for a synchronous session: the text is in the
    // field before the application sees the next key.
    const HRESULT hr = ctx->RequestEditSession(id, s, TF_ES_SYNC | TF_ES_READWRITE, &session_hr);
    s->Release();
    return FAILED(hr) ? hr : session_hr;
}

template <class T>
void safe_release(T*& p) {
    if (p) {
        p->Release();
        p = nullptr;
    }
}

// Read %APPDATA%\Buuz\settings.conf once, at activation. Empty when absent.
std::string load_config() {
    wchar_t dir[MAX_PATH];
    const DWORD n = GetEnvironmentVariableW(L"APPDATA", dir, MAX_PATH);
    if (n == 0 || n >= MAX_PATH) return {};
    std::wstring path = std::wstring(dir) + L"\\Buuz\\settings.conf";
    HANDLE f = CreateFileW(path.c_str(), GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING,
                           FILE_ATTRIBUTE_NORMAL, nullptr);
    if (f == INVALID_HANDLE_VALUE) return {};
    std::string text;
    char buf[4096];
    DWORD got = 0;
    while (text.size() < 65536 && ReadFile(f, buf, sizeof buf, &got, nullptr) && got > 0) {
        text.append(buf, got);
    }
    CloseHandle(f);
    return text;
}

// The word list is read from `mn.txt` in the folder of this DLL. It loads on
// the first typing key, once for each process. A missing or bad file gives
// no list: the engine then types without word choice.
const BuuzWordList* shared_word_list() {
    static std::once_flag once;
    static std::unique_ptr<WordList> list;
    std::call_once(once, [] {
        HMODULE module = nullptr;
        if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
                                    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                                reinterpret_cast<LPCWSTR>(&shared_word_list), &module)) {
            return;
        }
        wchar_t path[MAX_PATH];
        const DWORD n = GetModuleFileNameW(module, path, MAX_PATH);
        if (n == 0 || n >= MAX_PATH) return;
        std::wstring file(path, n);
        const size_t slash = file.find_last_of(L'\\');
        if (slash == std::wstring::npos) return;
        file.resize(slash + 1);
        file += L"mn.txt";
        HANDLE f = CreateFileW(file.c_str(), GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING,
                               FILE_ATTRIBUTE_NORMAL, nullptr);
        if (f == INVALID_HANDLE_VALUE) return;
        std::string text;
        char buf[65536];
        DWORD got = 0;
        // The list is under 1 MB. Stop at 16 MB if the file is not the list.
        while (text.size() < (16u << 20) && ReadFile(f, buf, sizeof buf, &got, nullptr) && got > 0) {
            text.append(buf, got);
        }
        CloseHandle(f);
        auto parsed = std::make_unique<WordList>(text);
        if (parsed->ok()) list = std::move(parsed);
    });
    return list ? list->get() : nullptr;
}

}  // namespace

TextService::TextService() { dll_add_ref(); }

TextService::~TextService() { dll_release(); }

STDMETHODIMP TextService::QueryInterface(REFIID riid, void** ppv) {
    if (!ppv) return E_INVALIDARG;
    *ppv = nullptr;
    if (riid == IID_IUnknown || riid == IID_ITfTextInputProcessor ||
        riid == IID_ITfTextInputProcessorEx) {
        *ppv = static_cast<ITfTextInputProcessorEx*>(this);
    } else if (riid == IID_ITfKeyEventSink) {
        *ppv = static_cast<ITfKeyEventSink*>(this);
    } else if (riid == IID_ITfCompositionSink) {
        *ppv = static_cast<ITfCompositionSink*>(this);
    } else if (riid == IID_ITfThreadMgrEventSink) {
        *ppv = static_cast<ITfThreadMgrEventSink*>(this);
    } else {
        return E_NOINTERFACE;
    }
    AddRef();
    return S_OK;
}

STDMETHODIMP_(ULONG) TextService::AddRef() { return InterlockedIncrement(&refs_); }

STDMETHODIMP_(ULONG) TextService::Release() {
    const LONG n = InterlockedDecrement(&refs_);
    if (n == 0) delete this;
    return static_cast<ULONG>(n);
}

STDMETHODIMP TextService::Activate(ITfThreadMgr* tm, TfClientId id) { return ActivateEx(tm, id, 0); }

STDMETHODIMP TextService::ActivateEx(ITfThreadMgr* tm, TfClientId id, DWORD) {
    if (!tm) return E_INVALIDARG;
    thread_mgr_ = tm;
    thread_mgr_->AddRef();
    client_id_ = id;

    engine_ = std::make_unique<Engine>(load_config(), shared_word_list);

    ITfCategoryMgr* cat = nullptr;
    if (SUCCEEDED(CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER,
                                   IID_ITfCategoryMgr, reinterpret_cast<void**>(&cat)))) {
        cat->RegisterGUID(GUID_BuuzDisplayAttribute, &attribute_atom_);
        cat->Release();
    }

    ITfKeystrokeMgr* keys = nullptr;
    HRESULT hr = thread_mgr_->QueryInterface(IID_ITfKeystrokeMgr, reinterpret_cast<void**>(&keys));
    if (SUCCEEDED(hr)) {
        hr = keys->AdviseKeyEventSink(client_id_, static_cast<ITfKeyEventSink*>(this), TRUE);
        keys->Release();
    }
    if (FAILED(hr)) {
        Deactivate();
        return hr;
    }

    ITfSource* source = nullptr;
    if (SUCCEEDED(thread_mgr_->QueryInterface(IID_ITfSource, reinterpret_cast<void**>(&source)))) {
        source->AdviseSink(IID_ITfThreadMgrEventSink, static_cast<ITfThreadMgrEventSink*>(this),
                           &thread_sink_cookie_);
        source->Release();
    }
    return S_OK;
}

STDMETHODIMP TextService::Deactivate() {
    if (thread_mgr_) {
        ITfSource* source = nullptr;
        if (thread_sink_cookie_ != TF_INVALID_COOKIE &&
            SUCCEEDED(thread_mgr_->QueryInterface(IID_ITfSource, reinterpret_cast<void**>(&source)))) {
            source->UnadviseSink(thread_sink_cookie_);
            source->Release();
        }
        thread_sink_cookie_ = TF_INVALID_COOKIE;
        ITfKeystrokeMgr* keys = nullptr;
        if (SUCCEEDED(thread_mgr_->QueryInterface(IID_ITfKeystrokeMgr, reinterpret_cast<void**>(&keys)))) {
            keys->UnadviseKeyEventSink(client_id_);
            keys->Release();
        }
    }
    commit_pending();
    engine_.reset();
    safe_release(thread_mgr_);
    client_id_ = TF_CLIENTID_NULL;
    return S_OK;
}

bool TextService::read_modifiers(Modifiers* mods) const {
    mods->shift = (GetKeyState(VK_SHIFT) & 0x8000) != 0;
    mods->caps = (GetKeyState(VK_CAPITAL) & 0x0001) != 0;
    // The Windows key counts as Ctrl: the chord belongs to the system.
    mods->ctrl = (GetKeyState(VK_CONTROL) & 0x8000) != 0 || (GetKeyState(VK_LWIN) & 0x8000) != 0 ||
                 (GetKeyState(VK_RWIN) & 0x8000) != 0;
    mods->alt = (GetKeyState(VK_MENU) & 0x8000) != 0;
    return true;
}

STDMETHODIMP TextService::OnSetFocus(BOOL foreground) {
    if (!foreground) commit_pending();
    return S_OK;
}

STDMETHODIMP TextService::OnSetFocus(ITfDocumentMgr*, ITfDocumentMgr*) {
    commit_pending();
    return S_OK;
}

STDMETHODIMP TextService::OnTestKeyDown(ITfContext*, WPARAM wp, LPARAM, BOOL* eaten) {
    if (!eaten) return E_INVALIDARG;
    *eaten = FALSE;
    if (!engine_ || !engine_->ok()) return S_OK;
    Modifiers mods;
    read_modifiers(&mods);
    const uint32_t vk = static_cast<uint32_t>(wp);
    // Say yes for every key the engine may use. OnKeyDown then decides, and
    // it may still let the key through.
    *eaten = engine_->composing() || is_typing_key(vk, mods) || vk == VK_BACK_ ? TRUE : FALSE;
    return S_OK;
}

STDMETHODIMP TextService::OnKeyDown(ITfContext* ctx, WPARAM wp, LPARAM, BOOL* eaten) {
    if (!eaten) return E_INVALIDARG;
    *eaten = FALSE;
    if (!ctx || !engine_ || !engine_->ok()) return S_OK;

    Modifiers mods;
    read_modifiers(&mods);
    const KeyInput key = classify(static_cast<uint32_t>(wp), mods);
    const Edit edit = engine_->process(key);
    *eaten = edit.consumed ? TRUE : FALSE;

    const EditPlan steps = plan(edit, composition_ != nullptr);
    if (!steps.touch_text) return S_OK;  // nothing to change in the text field
    const HRESULT hr = run_session(ctx, client_id_, [this, ctx, &steps](TfEditCookie ec) {
        return apply_edit(ec, ctx, steps);
    });
    if (FAILED(hr)) {
        // Fail open: let the application handle the key, drop our state.
        engine_->cancel();
        *eaten = FALSE;
    }
    return S_OK;
}

STDMETHODIMP TextService::OnTestKeyUp(ITfContext*, WPARAM, LPARAM, BOOL* eaten) {
    if (eaten) *eaten = FALSE;
    return S_OK;
}

STDMETHODIMP TextService::OnKeyUp(ITfContext*, WPARAM, LPARAM, BOOL* eaten) {
    if (eaten) *eaten = FALSE;
    return S_OK;
}

STDMETHODIMP TextService::OnPreservedKey(ITfContext*, REFGUID, BOOL* eaten) {
    if (eaten) *eaten = FALSE;
    return S_OK;
}

STDMETHODIMP TextService::OnCompositionTerminated(TfEditCookie, ITfComposition* comp) {
    // The application ended our composition (it lost focus or moved the caret).
    if (comp == composition_) {
        safe_release(composition_);
        safe_release(context_);
    }
    if (engine_) engine_->cancel();
    return S_OK;
}

void TextService::commit_pending() {
    if (!composition_) {
        if (engine_) engine_->cancel();
        return;
    }
    ITfContext* ctx = context_;
    if (ctx) {
        ctx->AddRef();
        // Leave the text as it is, only end the composition.
        run_session(ctx, client_id_, [this](TfEditCookie ec) {
            end_composition(ec);
            return S_OK;
        });
        ctx->Release();
    }
    safe_release(composition_);
    safe_release(context_);
    if (engine_) engine_->cancel();
}

void TextService::end_composition(TfEditCookie ec) {
    if (composition_) composition_->EndComposition(ec);
    safe_release(composition_);
    safe_release(context_);
}

HRESULT TextService::insert_at_selection(TfEditCookie ec, ITfContext* ctx, const std::u16string& text) {
    ITfInsertAtSelection* ins = nullptr;
    HRESULT hr = ctx->QueryInterface(IID_ITfInsertAtSelection, reinterpret_cast<void**>(&ins));
    if (FAILED(hr)) return hr;
    ITfRange* range = nullptr;
    hr = ins->InsertTextAtSelection(ec, 0, reinterpret_cast<const WCHAR*>(text.data()),
                                    static_cast<LONG>(text.size()), &range);
    ins->Release();
    safe_release(range);
    return hr;
}

HRESULT TextService::delete_before_caret(TfEditCookie ec, ITfContext* ctx, uint32_t count) {
    TF_SELECTION sel;
    ULONG fetched = 0;
    HRESULT hr = ctx->GetSelection(ec, TF_DEFAULT_SELECTION, 1, &sel, &fetched);
    if (FAILED(hr) || fetched != 1) return FAILED(hr) ? hr : E_FAIL;
    ITfRange* range = nullptr;
    hr = sel.range->Clone(&range);
    safe_release(sel.range);
    if (FAILED(hr)) return hr;
    LONG moved = 0;
    range->Collapse(ec, TF_ANCHOR_START);
    range->ShiftStart(ec, -static_cast<LONG>(count), &moved, nullptr);
    hr = range->SetText(ec, 0, L"", 0);
    range->Release();
    return hr;
}

HRESULT TextService::start_composition(TfEditCookie ec, ITfContext* ctx) {
    ITfInsertAtSelection* ins = nullptr;
    HRESULT hr = ctx->QueryInterface(IID_ITfInsertAtSelection, reinterpret_cast<void**>(&ins));
    if (FAILED(hr)) return hr;
    ITfRange* range = nullptr;
    hr = ins->InsertTextAtSelection(ec, TF_IAS_QUERYONLY, nullptr, 0, &range);
    ins->Release();
    if (FAILED(hr)) return hr;
    ITfContextComposition* cc = nullptr;
    hr = ctx->QueryInterface(IID_ITfContextComposition, reinterpret_cast<void**>(&cc));
    if (SUCCEEDED(hr)) {
        hr = cc->StartComposition(ec, range, static_cast<ITfCompositionSink*>(this), &composition_);
        cc->Release();
    }
    range->Release();
    if (SUCCEEDED(hr) && composition_) {
        context_ = ctx;
        context_->AddRef();
    }
    return hr;
}

void TextService::set_attribute(TfEditCookie ec, ITfContext* ctx, ITfRange* range) {
    if (attribute_atom_ == TF_INVALID_GUIDATOM) return;
    ITfProperty* prop = nullptr;
    if (FAILED(ctx->GetProperty(GUID_PROP_ATTRIBUTE, &prop))) return;
    VARIANT v;
    VariantInit(&v);
    v.vt = VT_I4;
    v.lVal = static_cast<LONG>(attribute_atom_);
    prop->SetValue(ec, range, &v);
    prop->Release();
}

HRESULT TextService::apply_edit(TfEditCookie ec, ITfContext* ctx, const EditPlan& steps) {
    HRESULT hr = S_OK;

    if (steps.delete_utf16 > 0) {
        hr = delete_before_caret(ec, ctx, steps.delete_utf16);
        if (FAILED(hr)) return hr;
    }

    if (!steps.commit.empty()) {
        if (composition_) {
            ITfRange* range = nullptr;
            hr = composition_->GetRange(&range);
            if (FAILED(hr)) return hr;
            hr = range->SetText(ec, 0, reinterpret_cast<const WCHAR*>(steps.commit.data()),
                                static_cast<LONG>(steps.commit.size()));
            if (SUCCEEDED(hr)) {
                // Caret after the committed word.
                range->Collapse(ec, TF_ANCHOR_END);
                TF_SELECTION sel = {range, {TF_AE_NONE, FALSE}};
                ctx->SetSelection(ec, 1, &sel);
            }
            range->Release();
            end_composition(ec);
            if (FAILED(hr)) return hr;
        } else {
            hr = insert_at_selection(ec, ctx, steps.commit);
            if (FAILED(hr)) return hr;
        }
    } else if (steps.end_composition && composition_) {
        // The composition ended with no text (Backspace on the last letter).
        ITfRange* range = nullptr;
        if (SUCCEEDED(composition_->GetRange(&range))) {
            range->SetText(ec, 0, L"", 0);
            range->Release();
        }
        end_composition(ec);
    }

    if (!steps.preedit.empty()) {
        if (steps.start_composition) {
            hr = start_composition(ec, ctx);
            if (FAILED(hr) || !composition_) return FAILED(hr) ? hr : E_FAIL;
        }
        if (!composition_) return E_FAIL;
        ITfRange* range = nullptr;
        hr = composition_->GetRange(&range);
        if (FAILED(hr)) return hr;
        hr = range->SetText(ec, 0, reinterpret_cast<const WCHAR*>(steps.preedit.data()),
                            static_cast<LONG>(steps.preedit.size()));
        if (SUCCEEDED(hr)) {
            set_attribute(ec, ctx, range);
            // Caret at the cursor of the preedit.
            ITfRange* caret = nullptr;
            if (SUCCEEDED(range->Clone(&caret))) {
                LONG moved = 0;
                caret->Collapse(ec, TF_ANCHOR_START);
                caret->ShiftEnd(ec, static_cast<LONG>(steps.caret_utf16), &moved, nullptr);
                caret->Collapse(ec, TF_ANCHOR_END);
                TF_SELECTION sel = {caret, {TF_AE_NONE, FALSE}};
                ctx->SetSelection(ec, 1, &sel);
                caret->Release();
            }
        }
        range->Release();
    }
    return hr;
}

}  // namespace buuz
