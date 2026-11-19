// SPDX-License-Identifier: MIT OR Apache-2.0
#include "display_attributes.h"

#include <new>

#include "guids.h"
#include "text_service.h"

namespace buuz {

namespace {

class AttributeInfo final : public ITfDisplayAttributeInfo {
public:
    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override {
        if (!ppv) return E_INVALIDARG;
        if (riid == IID_IUnknown || riid == IID_ITfDisplayAttributeInfo) {
            *ppv = static_cast<ITfDisplayAttributeInfo*>(this);
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
    STDMETHODIMP GetGUID(GUID* guid) override {
        if (!guid) return E_INVALIDARG;
        *guid = GUID_BuuzDisplayAttribute;
        return S_OK;
    }
    STDMETHODIMP GetDescription(BSTR* desc) override {
        if (!desc) return E_INVALIDARG;
        *desc = SysAllocString(L"Buuz");
        return *desc ? S_OK : E_OUTOFMEMORY;
    }
    STDMETHODIMP GetAttributeInfo(TF_DISPLAYATTRIBUTE* attr) override {
        if (!attr) return E_INVALIDARG;
        attr->crText = {TF_CT_NONE, {0}};
        attr->crBk = {TF_CT_NONE, {0}};
        attr->lsStyle = TF_LS_SOLID;
        attr->fBoldLine = FALSE;
        attr->crLine = {TF_CT_NONE, {0}};
        attr->bAttr = TF_ATTR_INPUT;
        return S_OK;
    }
    STDMETHODIMP SetAttributeInfo(const TF_DISPLAYATTRIBUTE*) override { return E_NOTIMPL; }
    STDMETHODIMP Reset() override { return S_OK; }

private:
    ULONG refs_ = 1;
};

class Enum final : public IEnumTfDisplayAttributeInfo {
public:
    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override {
        if (!ppv) return E_INVALIDARG;
        if (riid == IID_IUnknown || riid == IID_IEnumTfDisplayAttributeInfo) {
            *ppv = static_cast<IEnumTfDisplayAttributeInfo*>(this);
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
    STDMETHODIMP Clone(IEnumTfDisplayAttributeInfo** out) override {
        if (!out) return E_INVALIDARG;
        Enum* e = new (std::nothrow) Enum();
        if (!e) return E_OUTOFMEMORY;
        e->index_ = index_;
        *out = e;
        return S_OK;
    }
    STDMETHODIMP Next(ULONG count, ITfDisplayAttributeInfo** info, ULONG* fetched) override {
        ULONG got = 0;
        if (index_ == 0 && count > 0 && info) {
            AttributeInfo* a = new (std::nothrow) AttributeInfo();
            if (!a) return E_OUTOFMEMORY;
            info[0] = a;
            index_ = 1;
            got = 1;
        }
        if (fetched) *fetched = got;
        return got == count ? S_OK : S_FALSE;
    }
    STDMETHODIMP Reset() override {
        index_ = 0;
        return S_OK;
    }
    STDMETHODIMP Skip(ULONG count) override {
        index_ += count;
        return index_ > 1 ? S_FALSE : S_OK;
    }

private:
    ULONG refs_ = 1;
    ULONG index_ = 0;
};

class Provider final : public ITfDisplayAttributeProvider {
public:
    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override {
        if (!ppv) return E_INVALIDARG;
        if (riid == IID_IUnknown || riid == IID_ITfDisplayAttributeProvider) {
            *ppv = static_cast<ITfDisplayAttributeProvider*>(this);
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
    STDMETHODIMP EnumDisplayAttributeInfo(IEnumTfDisplayAttributeInfo** out) override {
        if (!out) return E_INVALIDARG;
        *out = new (std::nothrow) Enum();
        return *out ? S_OK : E_OUTOFMEMORY;
    }
    STDMETHODIMP GetDisplayAttributeInfo(REFGUID guid, ITfDisplayAttributeInfo** out) override {
        if (!out) return E_INVALIDARG;
        *out = nullptr;
        if (guid != GUID_BuuzDisplayAttribute) return E_INVALIDARG;
        *out = new (std::nothrow) AttributeInfo();
        return *out ? S_OK : E_OUTOFMEMORY;
    }

private:
    ULONG refs_ = 1;
};

}  // namespace

HRESULT create_display_attribute_provider(REFIID riid, void** ppv) {
    Provider* p = new (std::nothrow) Provider();
    if (!p) return E_OUTOFMEMORY;
    const HRESULT hr = p->QueryInterface(riid, ppv);
    p->Release();
    return hr;
}

}  // namespace buuz
