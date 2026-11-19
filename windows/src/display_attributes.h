// SPDX-License-Identifier: MIT OR Apache-2.0
// Tells TSF to draw the preedit with a solid underline. Windows only.
#pragma once

#include <msctf.h>
#include <ctfutb.h>

namespace buuz {

// TSF asks for this object through the CLSID of the text service, with
// IID_ITfDisplayAttributeProvider. The class factory hands it out.
HRESULT create_display_attribute_provider(REFIID riid, void** ppv);

}  // namespace buuz
