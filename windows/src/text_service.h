// SPDX-License-Identifier: MIT OR Apache-2.0
// The TSF text service. Windows only.
#pragma once

#include <msctf.h>

#include <memory>
#include <string>

#include "engine.h"

namespace buuz {

class DisplayAttributes;

// One instance per thread that types. Windows loads the DLL into each app.
class TextService final : public ITfTextInputProcessorEx,
                          public ITfKeyEventSink,
                          public ITfCompositionSink,
                          public ITfThreadMgrEventSink {
public:
    TextService();

    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override;
    STDMETHODIMP_(ULONG) AddRef() override;
    STDMETHODIMP_(ULONG) Release() override;

    // ITfTextInputProcessor(Ex)
    STDMETHODIMP Activate(ITfThreadMgr* tm, TfClientId id) override;
    STDMETHODIMP ActivateEx(ITfThreadMgr* tm, TfClientId id, DWORD flags) override;
    STDMETHODIMP Deactivate() override;

    // ITfKeyEventSink
    STDMETHODIMP OnSetFocus(BOOL foreground) override;
    STDMETHODIMP OnTestKeyDown(ITfContext* ctx, WPARAM wp, LPARAM lp, BOOL* eaten) override;
    STDMETHODIMP OnKeyDown(ITfContext* ctx, WPARAM wp, LPARAM lp, BOOL* eaten) override;
    STDMETHODIMP OnTestKeyUp(ITfContext* ctx, WPARAM wp, LPARAM lp, BOOL* eaten) override;
    STDMETHODIMP OnKeyUp(ITfContext* ctx, WPARAM wp, LPARAM lp, BOOL* eaten) override;
    STDMETHODIMP OnPreservedKey(ITfContext* ctx, REFGUID rguid, BOOL* eaten) override;

    // ITfCompositionSink
    STDMETHODIMP OnCompositionTerminated(TfEditCookie ec, ITfComposition* comp) override;

    // ITfThreadMgrEventSink: focus moves between documents end the word.
    STDMETHODIMP OnInitDocumentMgr(ITfDocumentMgr*) override { return S_OK; }
    STDMETHODIMP OnUninitDocumentMgr(ITfDocumentMgr*) override { return S_OK; }
    STDMETHODIMP OnSetFocus(ITfDocumentMgr* focus, ITfDocumentMgr* prev) override;
    STDMETHODIMP OnPushContext(ITfContext*) override { return S_OK; }
    STDMETHODIMP OnPopContext(ITfContext*) override { return S_OK; }

private:
    ~TextService();
    bool read_modifiers(Modifiers* mods) const;
    // The five steps of one key, always inside an edit session (`ec`).
    HRESULT apply_edit(TfEditCookie ec, ITfContext* ctx, const Edit& edit);
    HRESULT delete_before_caret(TfEditCookie ec, ITfContext* ctx, uint32_t count);
    HRESULT insert_at_selection(TfEditCookie ec, ITfContext* ctx, const std::u16string& text);
    HRESULT start_composition(TfEditCookie ec, ITfContext* ctx);
    void end_composition(TfEditCookie ec);
    void set_attribute(TfEditCookie ec, ITfContext* ctx, ITfRange* range);
    // Focus moved: end the composition and keep its text.
    void commit_pending();

    LONG refs_ = 1;
    ITfThreadMgr* thread_mgr_ = nullptr;
    TfClientId client_id_ = TF_CLIENTID_NULL;
    DWORD thread_sink_cookie_ = TF_INVALID_COOKIE;
    ITfComposition* composition_ = nullptr;
    ITfContext* context_ = nullptr;  // the context of composition_
    TfGuidAtom attribute_atom_ = TF_INVALID_GUIDATOM;
    std::unique_ptr<Engine> engine_;
};

// Class factory and registration, see registration.cpp.
HRESULT create_class_factory(REFIID riid, void** ppv);
HRESULT register_server(HMODULE module);
HRESULT unregister_server();
void dll_add_ref();
void dll_release();
bool dll_can_unload();

}  // namespace buuz
