#pragma once

#include "cxx.h"
#include "pro.h"
#include "typeinf.hpp"

#include <cstdint>
#include <memory>

// Parse types from a header file
inline int idalib_parse_header_file(const char *filename) {
  if (filename == nullptr) {
    return -1;
  }

  til_t *til = get_idati();
  if (til == nullptr) {
    return -1;
  }

  // HTI_FIL = input is filename, HTI_MAC = define macros from base tils,
  // HTI_NWR = no warnings
  int flags = HTI_FIL | HTI_MAC | HTI_NWR;
  return parse_decls(til, filename, nullptr, flags);
}

// Get type name from tinfo_t (using void* to avoid direct tinfo_t exposure)
inline rust::String idalib_tinfo_get_name_by_ordinal(std::uint32_t ordinal) {
  tinfo_t tif;

  if (!tif.get_numbered_type(get_idati(), ordinal)) {
    return rust::String();
  }

  const char *name = tif.dstr();
  if (name == nullptr) {
    return rust::String();
  }

  return rust::String(name);
}

// Check if a type ordinal is valid
inline bool idalib_is_valid_type_ordinal(std::uint32_t ordinal) {
  tinfo_t tif;
  return tif.get_numbered_type(get_idati(), ordinal);
}

// Get the maximum ordinal for type iteration
inline std::uint32_t idalib_get_type_ordinal_limit() {
  return get_ordinal_limit(get_idati());
}

// Apply type to an address using ordinal
inline bool idalib_apply_type_by_ordinal(std::uint64_t ea,
                                         std::uint32_t ordinal,
                                         std::uint32_t flags) {
  tinfo_t tif;

  if (!tif.get_numbered_type(get_idati(), ordinal)) {
    return false;
  }

  return apply_tinfo(ea, tif, flags);
}

// Apply type to an address using C declaration string
inline bool idalib_apply_type_by_decl(std::uint64_t ea, const char *decl) {
  if (decl == nullptr) {
    return false;
  }

  til_t *til = get_idati();
  if (til == nullptr) {
    return false;
  }

  return apply_cdecl(til, ea, decl);
}

// Get type information at an address (returns ordinal, 0 if no type)
inline std::uint32_t idalib_get_type_ordinal_at_address(std::uint64_t ea) {
  tinfo_t tif;

  if (!guess_tinfo(&tif, ea)) {
    return 0;
  }

  // Try to find the ordinal for this type
  std::uint32_t limit = get_ordinal_limit(get_idati());
  for (std::uint32_t i = 1; i < limit; i++) {
    tinfo_t check_tif;
    if (check_tif.get_numbered_type(get_idati(), i)) {
      if (tif.equals_to(check_tif)) {
        return i;
      }
    }
  }

  return 0; // Type not found in numbered types
}

// Get type declaration string at an address
inline rust::String idalib_get_type_string_at_address(std::uint64_t ea) {
  tinfo_t tif;

  if (!guess_tinfo(&tif, ea)) {
    return rust::String();
  }

  const char *type_str = tif.dstr();
  if (type_str == nullptr) {
    return rust::String();
  }

  return rust::String(type_str);
}

// ============================================================================
// Type Builder FFI Functions
// ============================================================================

// Create a simple primitive type ordinal
inline std::uint32_t idalib_create_primitive_type(std::uint32_t bt_type) {
  // For now, just return predefined ordinals for basic types
  // These are typically standard in IDA's type system
  switch (bt_type) {
    case 0x00: return 1;  // void
    case 0x01: return 2;  // int8
    case 0x02: return 3;  // int16
    case 0x03: return 4;  // int32
    case 0x04: return 5;  // int64
    case 0x05: return 6;  // uint8
    case 0x06: return 7;  // uint16
    case 0x07: return 8;  // uint32
    case 0x08: return 9;  // uint64
    case 0x09: return 10; // float
    case 0x0A: return 11; // double
    default: return 0;
  }
}