// SPDX-License-Identifier: MIT OR Apache-2.0
// Identifiers of the Buuz text service. Never change them: Windows stores
// the user's language list with these values.
#pragma once

#include <guiddef.h>

// {5E0B7A2C-9D41-4C7E-8B1A-3F6D2A9C4E10}
DEFINE_GUID(CLSID_BuuzTextService, 0x5e0b7a2c, 0x9d41, 0x4c7e, 0x8b, 0x1a, 0x3f, 0x6d, 0x2a, 0x9c, 0x4e, 0x10);
// {A83C1D55-2B6E-4F90-9E37-7C1B0D4F8A62}
DEFINE_GUID(GUID_BuuzProfile, 0xa83c1d55, 0x2b6e, 0x4f90, 0x9e, 0x37, 0x7c, 0x1b, 0x0d, 0x4f, 0x8a, 0x62);
// {3C9E5F71-6A2D-4B38-A0C4-18E7D5B2F936} underlined preedit
DEFINE_GUID(GUID_BuuzDisplayAttribute, 0x3c9e5f71, 0x6a2d, 0x4b38, 0xa0, 0xc4, 0x18, 0xe7, 0xd5, 0xb2, 0xf9, 0x36);

// Mongolian (Mongolia), Cyrillic: mn-MN.
constexpr unsigned short kLangMongolian = 0x0450;
