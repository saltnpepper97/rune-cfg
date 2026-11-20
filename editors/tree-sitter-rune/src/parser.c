#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 126
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 75
#define ALIAS_COUNT 0
#define TOKEN_COUNT 45
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 9
#define MAX_ALIAS_SEQUENCE_LENGTH 11
#define PRODUCTION_ID_COUNT 20

enum ts_symbol_identifiers {
  sym_identifier = 1,
  sym_comment = 2,
  anon_sym_AT = 3,
  anon_sym_gather = 4,
  anon_sym_as = 5,
  anon_sym_schema = 6,
  anon_sym_COLON = 7,
  anon_sym_end = 8,
  anon_sym_string = 9,
  anon_sym_str = 10,
  anon_sym_int = 11,
  anon_sym_integer = 12,
  anon_sym_float = 13,
  anon_sym_number = 14,
  anon_sym_bool = 15,
  anon_sym_boolean = 16,
  anon_sym_regex = 17,
  anon_sym_null = 18,
  anon_sym_any = 19,
  anon_sym_object = 20,
  anon_sym_enum = 21,
  anon_sym_LBRACK = 22,
  anon_sym_RBRACK = 23,
  anon_sym_required = 24,
  anon_sym_default = 25,
  anon_sym_range = 26,
  anon_sym_if = 27,
  anon_sym_elseif = 28,
  anon_sym_else_DASHif = 29,
  anon_sym_else = 30,
  anon_sym_EQ = 31,
  anon_sym_with = 32,
  sym_conditional_value = 33,
  anon_sym_COMMA = 34,
  sym_condition = 35,
  sym_reference = 36,
  sym_string = 37,
  sym_regex = 38,
  sym_number = 39,
  sym_range = 40,
  anon_sym_true = 41,
  anon_sym_false = 42,
  anon_sym_None = 43,
  sym__terminator = 44,
  sym_source_file = 45,
  sym__item = 46,
  sym_metadata = 47,
  sym_gather_statement = 48,
  sym_schema_block = 49,
  sym__schema_item = 50,
  sym_schema_object = 51,
  sym_schema_field = 52,
  sym__schema_type = 53,
  sym_schema_primitive = 54,
  sym_schema_enum = 55,
  sym_schema_array_type = 56,
  sym_schema_modifier = 57,
  sym_object_block = 58,
  sym__object_item = 59,
  sym_if_block = 60,
  sym_assignment = 61,
  sym_inline_attributes = 62,
  sym_inline_attribute = 63,
  sym__value = 64,
  sym_array = 65,
  sym_boolean = 66,
  sym_null = 67,
  aux_sym_source_file_repeat1 = 68,
  aux_sym_schema_block_repeat1 = 69,
  aux_sym_schema_field_repeat1 = 70,
  aux_sym_object_block_repeat1 = 71,
  aux_sym_if_block_repeat1 = 72,
  aux_sym_inline_attributes_repeat1 = 73,
  aux_sym_array_repeat1 = 74,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_identifier] = "identifier",
  [sym_comment] = "comment",
  [anon_sym_AT] = "@",
  [anon_sym_gather] = "gather",
  [anon_sym_as] = "as",
  [anon_sym_schema] = "schema",
  [anon_sym_COLON] = ":",
  [anon_sym_end] = "end",
  [anon_sym_string] = "string",
  [anon_sym_str] = "str",
  [anon_sym_int] = "int",
  [anon_sym_integer] = "integer",
  [anon_sym_float] = "float",
  [anon_sym_number] = "number",
  [anon_sym_bool] = "bool",
  [anon_sym_boolean] = "boolean",
  [anon_sym_regex] = "regex",
  [anon_sym_null] = "null",
  [anon_sym_any] = "any",
  [anon_sym_object] = "object",
  [anon_sym_enum] = "enum",
  [anon_sym_LBRACK] = "[",
  [anon_sym_RBRACK] = "]",
  [anon_sym_required] = "required",
  [anon_sym_default] = "default",
  [anon_sym_range] = "range",
  [anon_sym_if] = "if",
  [anon_sym_elseif] = "elseif",
  [anon_sym_else_DASHif] = "else-if",
  [anon_sym_else] = "else",
  [anon_sym_EQ] = "=",
  [anon_sym_with] = "with",
  [sym_conditional_value] = "conditional_value",
  [anon_sym_COMMA] = ",",
  [sym_condition] = "condition",
  [sym_reference] = "reference",
  [sym_string] = "string",
  [sym_regex] = "regex",
  [sym_number] = "number",
  [sym_range] = "range",
  [anon_sym_true] = "true",
  [anon_sym_false] = "false",
  [anon_sym_None] = "None",
  [sym__terminator] = "_terminator",
  [sym_source_file] = "source_file",
  [sym__item] = "_item",
  [sym_metadata] = "metadata",
  [sym_gather_statement] = "gather_statement",
  [sym_schema_block] = "schema_block",
  [sym__schema_item] = "_schema_item",
  [sym_schema_object] = "schema_object",
  [sym_schema_field] = "schema_field",
  [sym__schema_type] = "_schema_type",
  [sym_schema_primitive] = "schema_primitive",
  [sym_schema_enum] = "schema_enum",
  [sym_schema_array_type] = "schema_array_type",
  [sym_schema_modifier] = "schema_modifier",
  [sym_object_block] = "object_block",
  [sym__object_item] = "_object_item",
  [sym_if_block] = "if_block",
  [sym_assignment] = "assignment",
  [sym_inline_attributes] = "inline_attributes",
  [sym_inline_attribute] = "inline_attribute",
  [sym__value] = "_value",
  [sym_array] = "array",
  [sym_boolean] = "boolean",
  [sym_null] = "null",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_schema_block_repeat1] = "schema_block_repeat1",
  [aux_sym_schema_field_repeat1] = "schema_field_repeat1",
  [aux_sym_object_block_repeat1] = "object_block_repeat1",
  [aux_sym_if_block_repeat1] = "if_block_repeat1",
  [aux_sym_inline_attributes_repeat1] = "inline_attributes_repeat1",
  [aux_sym_array_repeat1] = "array_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_identifier] = sym_identifier,
  [sym_comment] = sym_comment,
  [anon_sym_AT] = anon_sym_AT,
  [anon_sym_gather] = anon_sym_gather,
  [anon_sym_as] = anon_sym_as,
  [anon_sym_schema] = anon_sym_schema,
  [anon_sym_COLON] = anon_sym_COLON,
  [anon_sym_end] = anon_sym_end,
  [anon_sym_string] = anon_sym_string,
  [anon_sym_str] = anon_sym_str,
  [anon_sym_int] = anon_sym_int,
  [anon_sym_integer] = anon_sym_integer,
  [anon_sym_float] = anon_sym_float,
  [anon_sym_number] = anon_sym_number,
  [anon_sym_bool] = anon_sym_bool,
  [anon_sym_boolean] = anon_sym_boolean,
  [anon_sym_regex] = anon_sym_regex,
  [anon_sym_null] = anon_sym_null,
  [anon_sym_any] = anon_sym_any,
  [anon_sym_object] = anon_sym_object,
  [anon_sym_enum] = anon_sym_enum,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [anon_sym_required] = anon_sym_required,
  [anon_sym_default] = anon_sym_default,
  [anon_sym_range] = anon_sym_range,
  [anon_sym_if] = anon_sym_if,
  [anon_sym_elseif] = anon_sym_elseif,
  [anon_sym_else_DASHif] = anon_sym_else_DASHif,
  [anon_sym_else] = anon_sym_else,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_with] = anon_sym_with,
  [sym_conditional_value] = sym_conditional_value,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [sym_condition] = sym_condition,
  [sym_reference] = sym_reference,
  [sym_string] = sym_string,
  [sym_regex] = sym_regex,
  [sym_number] = sym_number,
  [sym_range] = sym_range,
  [anon_sym_true] = anon_sym_true,
  [anon_sym_false] = anon_sym_false,
  [anon_sym_None] = anon_sym_None,
  [sym__terminator] = sym__terminator,
  [sym_source_file] = sym_source_file,
  [sym__item] = sym__item,
  [sym_metadata] = sym_metadata,
  [sym_gather_statement] = sym_gather_statement,
  [sym_schema_block] = sym_schema_block,
  [sym__schema_item] = sym__schema_item,
  [sym_schema_object] = sym_schema_object,
  [sym_schema_field] = sym_schema_field,
  [sym__schema_type] = sym__schema_type,
  [sym_schema_primitive] = sym_schema_primitive,
  [sym_schema_enum] = sym_schema_enum,
  [sym_schema_array_type] = sym_schema_array_type,
  [sym_schema_modifier] = sym_schema_modifier,
  [sym_object_block] = sym_object_block,
  [sym__object_item] = sym__object_item,
  [sym_if_block] = sym_if_block,
  [sym_assignment] = sym_assignment,
  [sym_inline_attributes] = sym_inline_attributes,
  [sym_inline_attribute] = sym_inline_attribute,
  [sym__value] = sym__value,
  [sym_array] = sym_array,
  [sym_boolean] = sym_boolean,
  [sym_null] = sym_null,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_schema_block_repeat1] = aux_sym_schema_block_repeat1,
  [aux_sym_schema_field_repeat1] = aux_sym_schema_field_repeat1,
  [aux_sym_object_block_repeat1] = aux_sym_object_block_repeat1,
  [aux_sym_if_block_repeat1] = aux_sym_if_block_repeat1,
  [aux_sym_inline_attributes_repeat1] = aux_sym_inline_attributes_repeat1,
  [aux_sym_array_repeat1] = aux_sym_array_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [sym_comment] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_AT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_gather] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_as] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_schema] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_end] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_string] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_str] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_int] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_integer] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_float] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_number] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_bool] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_boolean] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_regex] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_null] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_any] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_object] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_enum] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_required] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_default] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_range] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_if] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_elseif] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_else_DASHif] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_else] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_with] = {
    .visible = true,
    .named = false,
  },
  [sym_conditional_value] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [sym_condition] = {
    .visible = true,
    .named = true,
  },
  [sym_reference] = {
    .visible = true,
    .named = true,
  },
  [sym_string] = {
    .visible = true,
    .named = true,
  },
  [sym_regex] = {
    .visible = true,
    .named = true,
  },
  [sym_number] = {
    .visible = true,
    .named = true,
  },
  [sym_range] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_true] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_false] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_None] = {
    .visible = true,
    .named = false,
  },
  [sym__terminator] = {
    .visible = false,
    .named = true,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym__item] = {
    .visible = false,
    .named = true,
  },
  [sym_metadata] = {
    .visible = true,
    .named = true,
  },
  [sym_gather_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_schema_block] = {
    .visible = true,
    .named = true,
  },
  [sym__schema_item] = {
    .visible = false,
    .named = true,
  },
  [sym_schema_object] = {
    .visible = true,
    .named = true,
  },
  [sym_schema_field] = {
    .visible = true,
    .named = true,
  },
  [sym__schema_type] = {
    .visible = false,
    .named = true,
  },
  [sym_schema_primitive] = {
    .visible = true,
    .named = true,
  },
  [sym_schema_enum] = {
    .visible = true,
    .named = true,
  },
  [sym_schema_array_type] = {
    .visible = true,
    .named = true,
  },
  [sym_schema_modifier] = {
    .visible = true,
    .named = true,
  },
  [sym_object_block] = {
    .visible = true,
    .named = true,
  },
  [sym__object_item] = {
    .visible = false,
    .named = true,
  },
  [sym_if_block] = {
    .visible = true,
    .named = true,
  },
  [sym_assignment] = {
    .visible = true,
    .named = true,
  },
  [sym_inline_attributes] = {
    .visible = true,
    .named = true,
  },
  [sym_inline_attribute] = {
    .visible = true,
    .named = true,
  },
  [sym__value] = {
    .visible = false,
    .named = true,
  },
  [sym_array] = {
    .visible = true,
    .named = true,
  },
  [sym_boolean] = {
    .visible = true,
    .named = true,
  },
  [sym_null] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_schema_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_schema_field_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_object_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_if_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_inline_attributes_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_array_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_alias = 1,
  field_attributes = 2,
  field_condition = 3,
  field_end_name = 4,
  field_key = 5,
  field_name = 6,
  field_path = 7,
  field_type = 8,
  field_value = 9,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_alias] = "alias",
  [field_attributes] = "attributes",
  [field_condition] = "condition",
  [field_end_name] = "end_name",
  [field_key] = "key",
  [field_name] = "name",
  [field_path] = "path",
  [field_type] = "type",
  [field_value] = "value",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 1},
  [2] = {.index = 1, .length = 1},
  [3] = {.index = 2, .length = 2},
  [4] = {.index = 4, .length = 2},
  [5] = {.index = 6, .length = 1},
  [6] = {.index = 7, .length = 2},
  [7] = {.index = 9, .length = 3},
  [8] = {.index = 12, .length = 2},
  [9] = {.index = 14, .length = 1},
  [10] = {.index = 15, .length = 2},
  [11] = {.index = 17, .length = 3},
  [12] = {.index = 20, .length = 2},
  [13] = {.index = 22, .length = 2},
  [14] = {.index = 24, .length = 2},
  [15] = {.index = 26, .length = 2},
  [16] = {.index = 28, .length = 2},
  [17] = {.index = 30, .length = 2},
  [18] = {.index = 32, .length = 2},
  [19] = {.index = 34, .length = 2},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_name, 1},
  [1] =
    {field_path, 1},
  [2] =
    {field_key, 0},
    {field_value, 1},
  [4] =
    {field_name, 1},
    {field_value, 2},
  [6] =
    {field_name, 0},
  [7] =
    {field_key, 0},
    {field_value, 2},
  [9] =
    {field_attributes, 2},
    {field_key, 0},
    {field_value, 1},
  [12] =
    {field_alias, 3},
    {field_path, 1},
  [14] =
    {field_condition, 1},
  [15] =
    {field_end_name, 4},
    {field_name, 0},
  [17] =
    {field_attributes, 3},
    {field_key, 0},
    {field_value, 2},
  [20] =
    {field_name, 0},
    {field_value, 1},
  [22] =
    {field_end_name, 5},
    {field_name, 1},
  [24] =
    {field_condition, 1},
    {field_condition, 4, .inherited = true},
  [26] =
    {field_condition, 0, .inherited = true},
    {field_condition, 1, .inherited = true},
  [28] =
    {field_end_name, 5},
    {field_name, 0},
  [30] =
    {field_name, 0},
    {field_type, 1},
  [32] =
    {field_end_name, 6},
    {field_name, 1},
  [34] =
    {field_condition, 1},
    {field_condition, 5, .inherited = true},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 5,
  [7] = 7,
  [8] = 3,
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 54,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 22,
  [59] = 23,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 21,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 24,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 84,
  [85] = 85,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 95,
  [96] = 96,
  [97] = 97,
  [98] = 98,
  [99] = 99,
  [100] = 100,
  [101] = 101,
  [102] = 102,
  [103] = 103,
  [104] = 104,
  [105] = 105,
  [106] = 106,
  [107] = 107,
  [108] = 108,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 113,
  [114] = 114,
  [115] = 115,
  [116] = 116,
  [117] = 117,
  [118] = 118,
  [119] = 119,
  [120] = 120,
  [121] = 121,
  [122] = 122,
  [123] = 123,
  [124] = 124,
  [125] = 125,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(30);
      ADVANCE_MAP(
        '\n', 100,
        '"', 2,
        '#', 31,
        '\'', 6,
        ',', 46,
        ':', 33,
        '=', 38,
        '@', 32,
        'N', 80,
        '[', 36,
        ']', 37,
        'f', 72,
        'n', 83,
        'r', 71,
        't', 81,
        '+', 19,
        '-', 19,
      );
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(88);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 1:
      ADVANCE_MAP(
        '\n', 100,
        '"', 2,
        '#', 31,
        '$', 25,
        '\'', 6,
        ',', 46,
        ':', 33,
        '=', 38,
        'N', 59,
        '[', 36,
        ']', 37,
        'f', 50,
        'i', 54,
        'n', 62,
        'r', 49,
        't', 60,
        '+', 24,
        '-', 24,
      );
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(90);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(86);
      if (lookahead == '\\') ADVANCE(26);
      if (lookahead != 0) ADVANCE(2);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(87);
      if (lookahead == '\\') ADVANCE(28);
      if (lookahead != 0) ADVANCE(3);
      END_STATE();
    case 4:
      if (lookahead == '#') ADVANCE(31);
      if (lookahead == ':') ADVANCE(33);
      if (lookahead == '[') ADVANCE(36);
      if (lookahead == 'n') ADVANCE(83);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(4);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 5:
      if (lookahead == '#') ADVANCE(31);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(47);
      if (lookahead != 0 &&
          lookahead != '\t' &&
          lookahead != '\n' &&
          lookahead != ':') ADVANCE(48);
      END_STATE();
    case 6:
      if (lookahead == '\'') ADVANCE(86);
      if (lookahead == '\\') ADVANCE(27);
      if (lookahead != 0) ADVANCE(6);
      END_STATE();
    case 7:
      if (lookahead == '.') ADVANCE(18);
      END_STATE();
    case 8:
      if (lookahead == '.') ADVANCE(18);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(89);
      END_STATE();
    case 9:
      if (lookahead == '.') ADVANCE(18);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(10);
      END_STATE();
    case 10:
      if (lookahead == '.') ADVANCE(7);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(10);
      END_STATE();
    case 11:
      if (lookahead == '.') ADVANCE(9);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(11);
      END_STATE();
    case 12:
      if (lookahead == 'e') ADVANCE(65);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 13:
      if (lookahead == 'e') ADVANCE(13);
      if (lookahead == 'l') ADVANCE(14);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 14:
      if (lookahead == 'e') ADVANCE(13);
      if (lookahead == 's') ADVANCE(17);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 15:
      if (lookahead == 'e') ADVANCE(13);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 16:
      if (lookahead == 'e') ADVANCE(43);
      if (lookahead == 'l') ADVANCE(44);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(45);
      END_STATE();
    case 17:
      if (lookahead == 'e') ADVANCE(16);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 18:
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(20);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(92);
      END_STATE();
    case 19:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(88);
      END_STATE();
    case 20:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(92);
      END_STATE();
    case 21:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(93);
      END_STATE();
    case 22:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(91);
      END_STATE();
    case 23:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(11);
      END_STATE();
    case 24:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(90);
      END_STATE();
    case 25:
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 26:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(2);
      END_STATE();
    case 27:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(6);
      END_STATE();
    case 28:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(3);
      END_STATE();
    case 29:
      if (eof) ADVANCE(30);
      ADVANCE_MAP(
        '\n', 100,
        '"', 2,
        '#', 31,
        '\'', 6,
        '@', 32,
        ']', 37,
        '+', 23,
        '-', 23,
      );
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(29);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(11);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(31);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_AT);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(anon_sym_null);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 35:
      ACCEPT_TOKEN(anon_sym_null);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(sym_conditional_value);
      if (lookahead == '.') ADVANCE(42);
      if (lookahead == 'e') ADVANCE(39);
      if (lookahead == 'l') ADVANCE(40);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(41);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(45);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(sym_conditional_value);
      if (lookahead == '.') ADVANCE(42);
      if (lookahead == 'e') ADVANCE(39);
      if (lookahead == 's') ADVANCE(41);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(41);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(45);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(sym_conditional_value);
      if (lookahead == '.') ADVANCE(42);
      if (lookahead == 'e') ADVANCE(39);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(41);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(45);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(sym_conditional_value);
      if (lookahead == 'e') ADVANCE(39);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(41);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(45);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(sym_conditional_value);
      if (lookahead == 'e') ADVANCE(43);
      if (lookahead == 'l') ADVANCE(44);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(45);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(sym_conditional_value);
      if (lookahead == 'e') ADVANCE(43);
      if (lookahead == 's') ADVANCE(45);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(45);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(sym_conditional_value);
      if (lookahead == 'e') ADVANCE(43);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(45);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(sym_condition);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(47);
      if (lookahead != 0 &&
          lookahead != '\t' &&
          lookahead != '\n' &&
          lookahead != '#' &&
          lookahead != ':') ADVANCE(48);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(sym_condition);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#' &&
          lookahead != ':') ADVANCE(48);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '"') ADVANCE(3);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'a') ADVANCE(55);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'e') ADVANCE(98);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'e') ADVANCE(94);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'e') ADVANCE(96);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'f') ADVANCE(69);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'l') ADVANCE(61);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'l') ADVANCE(34);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'l') ADVANCE(56);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'n') ADVANCE(51);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'o') ADVANCE(58);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'r') ADVANCE(63);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 's') ADVANCE(53);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'u') ADVANCE(57);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == 'u') ADVANCE(52);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(12);
      if (lookahead == 'e') ADVANCE(65);
      if (lookahead == 'l') ADVANCE(66);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(12);
      if (lookahead == 'e') ADVANCE(65);
      if (lookahead == 's') ADVANCE(68);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(12);
      if (lookahead == 'e') ADVANCE(65);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(12);
      if (lookahead == 'e') ADVANCE(70);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(12);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(15);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(sym_reference);
      if (lookahead == '.') ADVANCE(42);
      if (lookahead == 'e') ADVANCE(39);
      if (lookahead == 'l') ADVANCE(40);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(41);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '#') ADVANCE(45);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '"') ADVANCE(3);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(76);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(99);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(95);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(97);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(82);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(35);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(77);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(73);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(79);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(84);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(75);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(78);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(74);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(sym_string);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(sym_regex);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(sym_number);
      if (lookahead == '.') ADVANCE(8);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(88);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(sym_number);
      if (lookahead == '.') ADVANCE(7);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(89);
      END_STATE();
    case 90:
      ACCEPT_TOKEN(sym_number);
      if (lookahead == '.') ADVANCE(22);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(90);
      END_STATE();
    case 91:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(91);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(sym_range);
      if (lookahead == '.') ADVANCE(21);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(92);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(sym_range);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(93);
      END_STATE();
    case 94:
      ACCEPT_TOKEN(anon_sym_true);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 95:
      ACCEPT_TOKEN(anon_sym_true);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 96:
      ACCEPT_TOKEN(anon_sym_false);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 97:
      ACCEPT_TOKEN(anon_sym_false);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 98:
      ACCEPT_TOKEN(anon_sym_None);
      if (lookahead == '.') ADVANCE(25);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      END_STATE();
    case 99:
      ACCEPT_TOKEN(anon_sym_None);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      END_STATE();
    case 100:
      ACCEPT_TOKEN(sym__terminator);
      if (lookahead == '\n') ADVANCE(100);
      END_STATE();
    default:
      return false;
  }
}

static bool ts_lex_keywords(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      ADVANCE_MAP(
        'a', 1,
        'b', 2,
        'd', 3,
        'e', 4,
        'f', 5,
        'g', 6,
        'i', 7,
        'n', 8,
        'o', 9,
        'r', 10,
        's', 11,
        'w', 12,
      );
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0);
      END_STATE();
    case 1:
      if (lookahead == 'n') ADVANCE(13);
      if (lookahead == 's') ADVANCE(14);
      END_STATE();
    case 2:
      if (lookahead == 'o') ADVANCE(15);
      END_STATE();
    case 3:
      if (lookahead == 'e') ADVANCE(16);
      END_STATE();
    case 4:
      if (lookahead == 'l') ADVANCE(17);
      if (lookahead == 'n') ADVANCE(18);
      END_STATE();
    case 5:
      if (lookahead == 'l') ADVANCE(19);
      END_STATE();
    case 6:
      if (lookahead == 'a') ADVANCE(20);
      END_STATE();
    case 7:
      if (lookahead == 'f') ADVANCE(21);
      if (lookahead == 'n') ADVANCE(22);
      END_STATE();
    case 8:
      if (lookahead == 'u') ADVANCE(23);
      END_STATE();
    case 9:
      if (lookahead == 'b') ADVANCE(24);
      END_STATE();
    case 10:
      if (lookahead == 'a') ADVANCE(25);
      if (lookahead == 'e') ADVANCE(26);
      END_STATE();
    case 11:
      if (lookahead == 'c') ADVANCE(27);
      if (lookahead == 't') ADVANCE(28);
      END_STATE();
    case 12:
      if (lookahead == 'i') ADVANCE(29);
      END_STATE();
    case 13:
      if (lookahead == 'y') ADVANCE(30);
      END_STATE();
    case 14:
      ACCEPT_TOKEN(anon_sym_as);
      END_STATE();
    case 15:
      if (lookahead == 'o') ADVANCE(31);
      END_STATE();
    case 16:
      if (lookahead == 'f') ADVANCE(32);
      END_STATE();
    case 17:
      if (lookahead == 's') ADVANCE(33);
      END_STATE();
    case 18:
      if (lookahead == 'd') ADVANCE(34);
      if (lookahead == 'u') ADVANCE(35);
      END_STATE();
    case 19:
      if (lookahead == 'o') ADVANCE(36);
      END_STATE();
    case 20:
      if (lookahead == 't') ADVANCE(37);
      END_STATE();
    case 21:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 22:
      if (lookahead == 't') ADVANCE(38);
      END_STATE();
    case 23:
      if (lookahead == 'm') ADVANCE(39);
      END_STATE();
    case 24:
      if (lookahead == 'j') ADVANCE(40);
      END_STATE();
    case 25:
      if (lookahead == 'n') ADVANCE(41);
      END_STATE();
    case 26:
      if (lookahead == 'g') ADVANCE(42);
      if (lookahead == 'q') ADVANCE(43);
      END_STATE();
    case 27:
      if (lookahead == 'h') ADVANCE(44);
      END_STATE();
    case 28:
      if (lookahead == 'r') ADVANCE(45);
      END_STATE();
    case 29:
      if (lookahead == 't') ADVANCE(46);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(anon_sym_any);
      END_STATE();
    case 31:
      if (lookahead == 'l') ADVANCE(47);
      END_STATE();
    case 32:
      if (lookahead == 'a') ADVANCE(48);
      END_STATE();
    case 33:
      if (lookahead == 'e') ADVANCE(49);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(anon_sym_end);
      END_STATE();
    case 35:
      if (lookahead == 'm') ADVANCE(50);
      END_STATE();
    case 36:
      if (lookahead == 'a') ADVANCE(51);
      END_STATE();
    case 37:
      if (lookahead == 'h') ADVANCE(52);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_int);
      if (lookahead == 'e') ADVANCE(53);
      END_STATE();
    case 39:
      if (lookahead == 'b') ADVANCE(54);
      END_STATE();
    case 40:
      if (lookahead == 'e') ADVANCE(55);
      END_STATE();
    case 41:
      if (lookahead == 'g') ADVANCE(56);
      END_STATE();
    case 42:
      if (lookahead == 'e') ADVANCE(57);
      END_STATE();
    case 43:
      if (lookahead == 'u') ADVANCE(58);
      END_STATE();
    case 44:
      if (lookahead == 'e') ADVANCE(59);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(anon_sym_str);
      if (lookahead == 'i') ADVANCE(60);
      END_STATE();
    case 46:
      if (lookahead == 'h') ADVANCE(61);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(anon_sym_bool);
      if (lookahead == 'e') ADVANCE(62);
      END_STATE();
    case 48:
      if (lookahead == 'u') ADVANCE(63);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(anon_sym_else);
      if (lookahead == '-') ADVANCE(64);
      if (lookahead == 'i') ADVANCE(65);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(anon_sym_enum);
      END_STATE();
    case 51:
      if (lookahead == 't') ADVANCE(66);
      END_STATE();
    case 52:
      if (lookahead == 'e') ADVANCE(67);
      END_STATE();
    case 53:
      if (lookahead == 'g') ADVANCE(68);
      END_STATE();
    case 54:
      if (lookahead == 'e') ADVANCE(69);
      END_STATE();
    case 55:
      if (lookahead == 'c') ADVANCE(70);
      END_STATE();
    case 56:
      if (lookahead == 'e') ADVANCE(71);
      END_STATE();
    case 57:
      if (lookahead == 'x') ADVANCE(72);
      END_STATE();
    case 58:
      if (lookahead == 'i') ADVANCE(73);
      END_STATE();
    case 59:
      if (lookahead == 'm') ADVANCE(74);
      END_STATE();
    case 60:
      if (lookahead == 'n') ADVANCE(75);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(anon_sym_with);
      END_STATE();
    case 62:
      if (lookahead == 'a') ADVANCE(76);
      END_STATE();
    case 63:
      if (lookahead == 'l') ADVANCE(77);
      END_STATE();
    case 64:
      if (lookahead == 'i') ADVANCE(78);
      END_STATE();
    case 65:
      if (lookahead == 'f') ADVANCE(79);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(anon_sym_float);
      END_STATE();
    case 67:
      if (lookahead == 'r') ADVANCE(80);
      END_STATE();
    case 68:
      if (lookahead == 'e') ADVANCE(81);
      END_STATE();
    case 69:
      if (lookahead == 'r') ADVANCE(82);
      END_STATE();
    case 70:
      if (lookahead == 't') ADVANCE(83);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(anon_sym_range);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(anon_sym_regex);
      END_STATE();
    case 73:
      if (lookahead == 'r') ADVANCE(84);
      END_STATE();
    case 74:
      if (lookahead == 'a') ADVANCE(85);
      END_STATE();
    case 75:
      if (lookahead == 'g') ADVANCE(86);
      END_STATE();
    case 76:
      if (lookahead == 'n') ADVANCE(87);
      END_STATE();
    case 77:
      if (lookahead == 't') ADVANCE(88);
      END_STATE();
    case 78:
      if (lookahead == 'f') ADVANCE(89);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(anon_sym_elseif);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(anon_sym_gather);
      END_STATE();
    case 81:
      if (lookahead == 'r') ADVANCE(90);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(anon_sym_number);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(anon_sym_object);
      END_STATE();
    case 84:
      if (lookahead == 'e') ADVANCE(91);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(anon_sym_schema);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(anon_sym_string);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(anon_sym_boolean);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(anon_sym_default);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(anon_sym_else_DASHif);
      END_STATE();
    case 90:
      ACCEPT_TOKEN(anon_sym_integer);
      END_STATE();
    case 91:
      if (lookahead == 'd') ADVANCE(92);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(anon_sym_required);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 29},
  [2] = {.lex_state = 4},
  [3] = {.lex_state = 1},
  [4] = {.lex_state = 1},
  [5] = {.lex_state = 1},
  [6] = {.lex_state = 1},
  [7] = {.lex_state = 4},
  [8] = {.lex_state = 1},
  [9] = {.lex_state = 29},
  [10] = {.lex_state = 1},
  [11] = {.lex_state = 29},
  [12] = {.lex_state = 1},
  [13] = {.lex_state = 1},
  [14] = {.lex_state = 29},
  [15] = {.lex_state = 1},
  [16] = {.lex_state = 1},
  [17] = {.lex_state = 1},
  [18] = {.lex_state = 29},
  [19] = {.lex_state = 29},
  [20] = {.lex_state = 29},
  [21] = {.lex_state = 1},
  [22] = {.lex_state = 1},
  [23] = {.lex_state = 1},
  [24] = {.lex_state = 1},
  [25] = {.lex_state = 29},
  [26] = {.lex_state = 29},
  [27] = {.lex_state = 29},
  [28] = {.lex_state = 29},
  [29] = {.lex_state = 29},
  [30] = {.lex_state = 29},
  [31] = {.lex_state = 29},
  [32] = {.lex_state = 29},
  [33] = {.lex_state = 29},
  [34] = {.lex_state = 29},
  [35] = {.lex_state = 29},
  [36] = {.lex_state = 29},
  [37] = {.lex_state = 29},
  [38] = {.lex_state = 29},
  [39] = {.lex_state = 29},
  [40] = {.lex_state = 29},
  [41] = {.lex_state = 29},
  [42] = {.lex_state = 29},
  [43] = {.lex_state = 29},
  [44] = {.lex_state = 29},
  [45] = {.lex_state = 29},
  [46] = {.lex_state = 29},
  [47] = {.lex_state = 29},
  [48] = {.lex_state = 29},
  [49] = {.lex_state = 29},
  [50] = {.lex_state = 29},
  [51] = {.lex_state = 29},
  [52] = {.lex_state = 29},
  [53] = {.lex_state = 29},
  [54] = {.lex_state = 29},
  [55] = {.lex_state = 29},
  [56] = {.lex_state = 29},
  [57] = {.lex_state = 29},
  [58] = {.lex_state = 29},
  [59] = {.lex_state = 29},
  [60] = {.lex_state = 29},
  [61] = {.lex_state = 29},
  [62] = {.lex_state = 29},
  [63] = {.lex_state = 29},
  [64] = {.lex_state = 29},
  [65] = {.lex_state = 29},
  [66] = {.lex_state = 29},
  [67] = {.lex_state = 29},
  [68] = {.lex_state = 29},
  [69] = {.lex_state = 29},
  [70] = {.lex_state = 29},
  [71] = {.lex_state = 29},
  [72] = {.lex_state = 29},
  [73] = {.lex_state = 29},
  [74] = {.lex_state = 29},
  [75] = {.lex_state = 29},
  [76] = {.lex_state = 29},
  [77] = {.lex_state = 29},
  [78] = {.lex_state = 29},
  [79] = {.lex_state = 29},
  [80] = {.lex_state = 29},
  [81] = {.lex_state = 29},
  [82] = {.lex_state = 29},
  [83] = {.lex_state = 29},
  [84] = {.lex_state = 29},
  [85] = {.lex_state = 29},
  [86] = {.lex_state = 29},
  [87] = {.lex_state = 29},
  [88] = {.lex_state = 29},
  [89] = {.lex_state = 29},
  [90] = {.lex_state = 29},
  [91] = {.lex_state = 29},
  [92] = {.lex_state = 29},
  [93] = {.lex_state = 29},
  [94] = {.lex_state = 29},
  [95] = {.lex_state = 0},
  [96] = {.lex_state = 29},
  [97] = {.lex_state = 5},
  [98] = {.lex_state = 0},
  [99] = {.lex_state = 0},
  [100] = {.lex_state = 0},
  [101] = {.lex_state = 0},
  [102] = {.lex_state = 0},
  [103] = {.lex_state = 0},
  [104] = {.lex_state = 0},
  [105] = {.lex_state = 0},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 0},
  [108] = {.lex_state = 0},
  [109] = {.lex_state = 0},
  [110] = {.lex_state = 29},
  [111] = {.lex_state = 0},
  [112] = {.lex_state = 0},
  [113] = {.lex_state = 0},
  [114] = {.lex_state = 29},
  [115] = {.lex_state = 5},
  [116] = {.lex_state = 0},
  [117] = {.lex_state = 0},
  [118] = {.lex_state = 29},
  [119] = {.lex_state = 0},
  [120] = {.lex_state = 0},
  [121] = {.lex_state = 0},
  [122] = {.lex_state = 29},
  [123] = {.lex_state = 0},
  [124] = {.lex_state = 0},
  [125] = {.lex_state = 0},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_identifier] = ACTIONS(1),
    [sym_comment] = ACTIONS(3),
    [anon_sym_AT] = ACTIONS(1),
    [anon_sym_gather] = ACTIONS(1),
    [anon_sym_as] = ACTIONS(1),
    [anon_sym_schema] = ACTIONS(1),
    [anon_sym_COLON] = ACTIONS(1),
    [anon_sym_end] = ACTIONS(1),
    [anon_sym_string] = ACTIONS(1),
    [anon_sym_str] = ACTIONS(1),
    [anon_sym_int] = ACTIONS(1),
    [anon_sym_integer] = ACTIONS(1),
    [anon_sym_float] = ACTIONS(1),
    [anon_sym_number] = ACTIONS(1),
    [anon_sym_bool] = ACTIONS(1),
    [anon_sym_boolean] = ACTIONS(1),
    [anon_sym_regex] = ACTIONS(1),
    [anon_sym_null] = ACTIONS(1),
    [anon_sym_any] = ACTIONS(1),
    [anon_sym_object] = ACTIONS(1),
    [anon_sym_enum] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_required] = ACTIONS(1),
    [anon_sym_default] = ACTIONS(1),
    [anon_sym_range] = ACTIONS(1),
    [anon_sym_if] = ACTIONS(1),
    [anon_sym_elseif] = ACTIONS(1),
    [anon_sym_else_DASHif] = ACTIONS(1),
    [anon_sym_else] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_with] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [sym_string] = ACTIONS(1),
    [sym_regex] = ACTIONS(1),
    [sym_number] = ACTIONS(1),
    [sym_range] = ACTIONS(1),
    [anon_sym_true] = ACTIONS(1),
    [anon_sym_false] = ACTIONS(1),
    [anon_sym_None] = ACTIONS(1),
    [sym__terminator] = ACTIONS(1),
  },
  [1] = {
    [sym_source_file] = STATE(109),
    [sym__item] = STATE(11),
    [sym_metadata] = STATE(11),
    [sym_gather_statement] = STATE(11),
    [sym_schema_block] = STATE(11),
    [sym_object_block] = STATE(11),
    [sym_if_block] = STATE(11),
    [sym_assignment] = STATE(11),
    [aux_sym_source_file_repeat1] = STATE(11),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_identifier] = ACTIONS(7),
    [sym_comment] = ACTIONS(3),
    [anon_sym_AT] = ACTIONS(9),
    [anon_sym_gather] = ACTIONS(11),
    [anon_sym_schema] = ACTIONS(13),
    [anon_sym_if] = ACTIONS(15),
    [sym_string] = ACTIONS(17),
    [sym__terminator] = ACTIONS(19),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(21), 1,
      anon_sym_COLON,
    ACTIONS(27), 1,
      anon_sym_enum,
    ACTIONS(29), 1,
      anon_sym_LBRACK,
    ACTIONS(25), 3,
      anon_sym_str,
      anon_sym_int,
      anon_sym_bool,
    STATE(72), 4,
      sym__schema_type,
      sym_schema_primitive,
      sym_schema_enum,
      sym_schema_array_type,
    ACTIONS(23), 9,
      anon_sym_string,
      anon_sym_integer,
      anon_sym_float,
      anon_sym_number,
      anon_sym_boolean,
      anon_sym_regex,
      anon_sym_null,
      anon_sym_any,
      anon_sym_object,
  [35] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(33), 1,
      anon_sym_LBRACK,
    ACTIONS(35), 1,
      anon_sym_RBRACK,
    ACTIONS(39), 1,
      sym_reference,
    ACTIONS(31), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(41), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(5), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(37), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [71] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(46), 1,
      anon_sym_LBRACK,
    ACTIONS(49), 1,
      anon_sym_RBRACK,
    ACTIONS(54), 1,
      sym_reference,
    ACTIONS(43), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(57), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(4), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(51), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [107] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(33), 1,
      anon_sym_LBRACK,
    ACTIONS(60), 1,
      anon_sym_RBRACK,
    ACTIONS(64), 1,
      sym_reference,
    ACTIONS(31), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(41), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(4), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(62), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [143] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(33), 1,
      anon_sym_LBRACK,
    ACTIONS(64), 1,
      sym_reference,
    ACTIONS(66), 1,
      anon_sym_RBRACK,
    ACTIONS(31), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(41), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(4), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(62), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [179] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(27), 1,
      anon_sym_enum,
    ACTIONS(29), 1,
      anon_sym_LBRACK,
    ACTIONS(25), 3,
      anon_sym_str,
      anon_sym_int,
      anon_sym_bool,
    STATE(105), 4,
      sym__schema_type,
      sym_schema_primitive,
      sym_schema_enum,
      sym_schema_array_type,
    ACTIONS(23), 9,
      anon_sym_string,
      anon_sym_integer,
      anon_sym_float,
      anon_sym_number,
      anon_sym_boolean,
      anon_sym_regex,
      anon_sym_null,
      anon_sym_any,
      anon_sym_object,
  [211] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(33), 1,
      anon_sym_LBRACK,
    ACTIONS(68), 1,
      anon_sym_RBRACK,
    ACTIONS(72), 1,
      sym_reference,
    ACTIONS(31), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(41), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(6), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(70), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [247] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(74), 1,
      ts_builtin_sym_end,
    ACTIONS(76), 1,
      sym_identifier,
    ACTIONS(79), 1,
      anon_sym_AT,
    ACTIONS(82), 1,
      anon_sym_gather,
    ACTIONS(85), 1,
      anon_sym_schema,
    ACTIONS(88), 1,
      anon_sym_if,
    ACTIONS(91), 1,
      sym_string,
    ACTIONS(94), 1,
      sym__terminator,
    STATE(9), 8,
      sym__item,
      sym_metadata,
      sym_gather_statement,
      sym_schema_block,
      sym_object_block,
      sym_if_block,
      sym_assignment,
      aux_sym_source_file_repeat1,
  [285] = 9,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(97), 1,
      anon_sym_COLON,
    ACTIONS(101), 1,
      anon_sym_LBRACK,
    ACTIONS(103), 1,
      anon_sym_EQ,
    ACTIONS(107), 1,
      sym_reference,
    ACTIONS(99), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(109), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(105), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(91), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [321] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(9), 1,
      anon_sym_AT,
    ACTIONS(11), 1,
      anon_sym_gather,
    ACTIONS(13), 1,
      anon_sym_schema,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(111), 1,
      ts_builtin_sym_end,
    ACTIONS(113), 1,
      sym__terminator,
    STATE(9), 8,
      sym__item,
      sym_metadata,
      sym_gather_statement,
      sym_schema_block,
      sym_object_block,
      sym_if_block,
      sym_assignment,
      aux_sym_source_file_repeat1,
  [359] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_LBRACK,
    ACTIONS(117), 1,
      sym_reference,
    ACTIONS(119), 1,
      sym__terminator,
    ACTIONS(99), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(109), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(115), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(121), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [392] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_LBRACK,
    ACTIONS(103), 1,
      anon_sym_EQ,
    ACTIONS(107), 1,
      sym_reference,
    ACTIONS(99), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(109), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(105), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(91), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [425] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(121), 1,
      anon_sym_end,
    ACTIONS(125), 1,
      anon_sym_else,
    ACTIONS(127), 1,
      sym__terminator,
    STATE(80), 1,
      aux_sym_if_block_repeat1,
    ACTIONS(123), 2,
      anon_sym_elseif,
      anon_sym_else_DASHif,
    STATE(18), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [461] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_LBRACK,
    ACTIONS(131), 1,
      sym_reference,
    ACTIONS(99), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(109), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(129), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(89), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [491] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_LBRACK,
    ACTIONS(135), 1,
      sym_reference,
    ACTIONS(99), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(109), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(133), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(86), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [521] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_LBRACK,
    ACTIONS(139), 1,
      sym_reference,
    ACTIONS(99), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(109), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(137), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(84), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [551] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(141), 1,
      anon_sym_end,
    ACTIONS(143), 1,
      anon_sym_else,
    ACTIONS(145), 1,
      sym__terminator,
    STATE(76), 1,
      aux_sym_if_block_repeat1,
    ACTIONS(123), 2,
      anon_sym_elseif,
      anon_sym_else_DASHif,
    STATE(20), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [587] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(149), 1,
      sym__terminator,
    ACTIONS(147), 4,
      anon_sym_end,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [616] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(151), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(159), 1,
      sym_string,
    ACTIONS(162), 1,
      sym__terminator,
    ACTIONS(154), 4,
      anon_sym_end,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
    STATE(20), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [645] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(165), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(167), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [666] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(169), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(171), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [687] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(173), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(175), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [708] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(177), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(179), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [729] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(145), 1,
      sym__terminator,
    ACTIONS(181), 4,
      anon_sym_end,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
    STATE(20), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [758] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(183), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(185), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [778] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(187), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(189), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [798] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(191), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(193), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [818] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(197), 1,
      sym_identifier,
    ACTIONS(195), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(199), 7,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
  [840] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(203), 1,
      sym_identifier,
    ACTIONS(201), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(205), 7,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
  [862] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(207), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(209), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [882] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(211), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(213), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [902] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(215), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(217), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [922] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(219), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(221), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [942] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(223), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(225), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [962] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(227), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(229), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [982] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(231), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(233), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [1002] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(235), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(237), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [1022] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(239), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(241), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [1042] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(243), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(245), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [1062] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(247), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(249), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [1082] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(251), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(253), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [1102] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(255), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(257), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [1122] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(259), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(261), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_else,
      sym_identifier,
  [1142] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(263), 1,
      anon_sym_end,
    ACTIONS(265), 1,
      sym__terminator,
    STATE(46), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1168] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(145), 1,
      sym__terminator,
    ACTIONS(267), 1,
      anon_sym_end,
    STATE(20), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1194] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(269), 1,
      anon_sym_end,
    ACTIONS(271), 1,
      sym__terminator,
    STATE(48), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1220] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(145), 1,
      sym__terminator,
    ACTIONS(273), 1,
      anon_sym_end,
    STATE(20), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1246] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(273), 1,
      anon_sym_end,
    ACTIONS(275), 1,
      sym__terminator,
    STATE(52), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1272] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(277), 1,
      anon_sym_end,
    ACTIONS(279), 1,
      sym__terminator,
    STATE(54), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1298] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(145), 1,
      sym__terminator,
    ACTIONS(281), 1,
      anon_sym_end,
    STATE(20), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1324] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(145), 1,
      sym__terminator,
    ACTIONS(283), 1,
      anon_sym_end,
    STATE(20), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1350] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(285), 1,
      anon_sym_end,
    ACTIONS(287), 1,
      sym__terminator,
    STATE(51), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1376] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(145), 1,
      sym__terminator,
    ACTIONS(289), 1,
      anon_sym_end,
    STATE(20), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1402] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(291), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(293), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1418] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(295), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(297), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1434] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(301), 1,
      sym_identifier,
    ACTIONS(303), 3,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
    ACTIONS(299), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
  [1452] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(171), 3,
      anon_sym_RBRACK,
      sym_string,
      sym__terminator,
    ACTIONS(169), 5,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      anon_sym_with,
      sym_identifier,
  [1468] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(175), 3,
      anon_sym_RBRACK,
      sym_string,
      sym__terminator,
    ACTIONS(173), 5,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      anon_sym_with,
      sym_identifier,
  [1484] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(305), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(307), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1500] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(311), 1,
      sym_identifier,
    ACTIONS(313), 3,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
    ACTIONS(309), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
  [1518] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(315), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(317), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1534] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(319), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(321), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1550] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(323), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(325), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1566] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(327), 1,
      sym_identifier,
    ACTIONS(330), 1,
      anon_sym_end,
    ACTIONS(332), 1,
      sym__terminator,
    STATE(65), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1585] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(167), 2,
      sym_string,
      sym__terminator,
    ACTIONS(165), 5,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      anon_sym_with,
      sym_identifier,
  [1600] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(335), 1,
      sym_identifier,
    ACTIONS(337), 1,
      anon_sym_end,
    ACTIONS(339), 1,
      sym__terminator,
    STATE(69), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1619] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(335), 1,
      sym_identifier,
    ACTIONS(341), 1,
      anon_sym_end,
    ACTIONS(343), 1,
      sym__terminator,
    STATE(65), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1638] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(335), 1,
      sym_identifier,
    ACTIONS(343), 1,
      sym__terminator,
    ACTIONS(345), 1,
      anon_sym_end,
    STATE(65), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1657] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(179), 2,
      sym_string,
      sym__terminator,
    ACTIONS(177), 5,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      anon_sym_with,
      sym_identifier,
  [1672] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(335), 1,
      sym_identifier,
    ACTIONS(347), 1,
      anon_sym_end,
    ACTIONS(349), 1,
      sym__terminator,
    STATE(68), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1691] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(351), 1,
      anon_sym_required,
    ACTIONS(353), 1,
      anon_sym_default,
    ACTIONS(355), 1,
      anon_sym_range,
    ACTIONS(357), 1,
      sym__terminator,
    STATE(73), 2,
      sym_schema_modifier,
      aux_sym_schema_field_repeat1,
  [1711] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(351), 1,
      anon_sym_required,
    ACTIONS(353), 1,
      anon_sym_default,
    ACTIONS(355), 1,
      anon_sym_range,
    ACTIONS(359), 1,
      sym__terminator,
    STATE(74), 2,
      sym_schema_modifier,
      aux_sym_schema_field_repeat1,
  [1731] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(361), 1,
      anon_sym_required,
    ACTIONS(364), 1,
      anon_sym_default,
    ACTIONS(367), 1,
      anon_sym_range,
    ACTIONS(370), 1,
      sym__terminator,
    STATE(74), 2,
      sym_schema_modifier,
      aux_sym_schema_field_repeat1,
  [1751] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(372), 5,
      anon_sym_RBRACK,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1762] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(374), 1,
      anon_sym_end,
    ACTIONS(378), 1,
      anon_sym_else,
    STATE(77), 1,
      aux_sym_if_block_repeat1,
    ACTIONS(376), 2,
      anon_sym_elseif,
      anon_sym_else_DASHif,
  [1779] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(380), 1,
      anon_sym_end,
    ACTIONS(385), 1,
      anon_sym_else,
    STATE(77), 1,
      aux_sym_if_block_repeat1,
    ACTIONS(382), 2,
      anon_sym_elseif,
      anon_sym_else_DASHif,
  [1796] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(390), 1,
      sym__terminator,
    ACTIONS(387), 2,
      sym_identifier,
      sym_string,
    STATE(78), 2,
      sym_inline_attribute,
      aux_sym_inline_attributes_repeat1,
  [1811] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(392), 5,
      anon_sym_RBRACK,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1822] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(394), 1,
      anon_sym_end,
    ACTIONS(396), 1,
      anon_sym_else,
    STATE(77), 1,
      aux_sym_if_block_repeat1,
    ACTIONS(376), 2,
      anon_sym_elseif,
      anon_sym_else_DASHif,
  [1839] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(398), 5,
      anon_sym_RBRACK,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1850] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(402), 1,
      sym__terminator,
    ACTIONS(400), 2,
      sym_identifier,
      sym_string,
    STATE(78), 2,
      sym_inline_attribute,
      aux_sym_inline_attributes_repeat1,
  [1865] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(400), 2,
      sym_identifier,
      sym_string,
    STATE(82), 2,
      sym_inline_attribute,
      aux_sym_inline_attributes_repeat1,
  [1877] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(404), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1887] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(406), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1897] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(408), 3,
      sym_identifier,
      sym_string,
      sym__terminator,
  [1906] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(412), 1,
      sym__terminator,
    ACTIONS(410), 2,
      anon_sym_end,
      sym_identifier,
  [1917] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(414), 1,
      sym_identifier,
    ACTIONS(416), 1,
      anon_sym_end,
    ACTIONS(418), 1,
      sym__terminator,
  [1930] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(420), 1,
      anon_sym_with,
    ACTIONS(422), 1,
      sym__terminator,
    STATE(119), 1,
      sym_inline_attributes,
  [1943] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(426), 1,
      sym__terminator,
    ACTIONS(424), 2,
      anon_sym_end,
      sym_identifier,
  [1954] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(420), 1,
      anon_sym_with,
    ACTIONS(428), 1,
      sym__terminator,
    STATE(106), 1,
      sym_inline_attributes,
  [1967] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(432), 1,
      sym__terminator,
    ACTIONS(430), 2,
      anon_sym_end,
      sym_identifier,
  [1978] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(434), 1,
      sym_identifier,
    ACTIONS(436), 1,
      anon_sym_end,
    ACTIONS(438), 1,
      sym__terminator,
  [1991] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(442), 1,
      sym__terminator,
    ACTIONS(440), 2,
      anon_sym_end,
      sym_identifier,
  [2002] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_LBRACK,
    STATE(79), 1,
      sym_array,
  [2012] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(444), 1,
      anon_sym_as,
    ACTIONS(446), 1,
      sym__terminator,
  [2022] = 2,
    ACTIONS(448), 1,
      sym_comment,
    ACTIONS(450), 1,
      sym_condition,
  [2029] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(452), 1,
      sym__terminator,
  [2036] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(454), 1,
      anon_sym_COLON,
  [2043] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(456), 1,
      anon_sym_COLON,
  [2050] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(458), 1,
      anon_sym_COLON,
  [2057] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(460), 1,
      sym__terminator,
  [2064] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(462), 1,
      sym__terminator,
  [2071] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(464), 1,
      sym__terminator,
  [2078] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(466), 1,
      anon_sym_RBRACK,
  [2085] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(468), 1,
      sym__terminator,
  [2092] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(470), 1,
      sym__terminator,
  [2099] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(472), 1,
      anon_sym_COLON,
  [2106] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(474), 1,
      ts_builtin_sym_end,
  [2113] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(476), 1,
      sym_identifier,
  [2120] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(478), 1,
      anon_sym_COLON,
  [2127] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(480), 1,
      sym_string,
  [2134] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(482), 1,
      sym__terminator,
  [2141] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(484), 1,
      sym_identifier,
  [2148] = 2,
    ACTIONS(448), 1,
      sym_comment,
    ACTIONS(486), 1,
      sym_condition,
  [2155] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(488), 1,
      anon_sym_COLON,
  [2162] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(490), 1,
      sym__terminator,
  [2169] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(137), 1,
      sym_range,
  [2176] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(492), 1,
      sym__terminator,
  [2183] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(494), 1,
      anon_sym_COLON,
  [2190] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(496), 1,
      sym__terminator,
  [2197] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(498), 1,
      sym_identifier,
  [2204] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(500), 1,
      sym__terminator,
  [2211] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(502), 1,
      sym__terminator,
  [2218] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(504), 1,
      sym__terminator,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 35,
  [SMALL_STATE(4)] = 71,
  [SMALL_STATE(5)] = 107,
  [SMALL_STATE(6)] = 143,
  [SMALL_STATE(7)] = 179,
  [SMALL_STATE(8)] = 211,
  [SMALL_STATE(9)] = 247,
  [SMALL_STATE(10)] = 285,
  [SMALL_STATE(11)] = 321,
  [SMALL_STATE(12)] = 359,
  [SMALL_STATE(13)] = 392,
  [SMALL_STATE(14)] = 425,
  [SMALL_STATE(15)] = 461,
  [SMALL_STATE(16)] = 491,
  [SMALL_STATE(17)] = 521,
  [SMALL_STATE(18)] = 551,
  [SMALL_STATE(19)] = 587,
  [SMALL_STATE(20)] = 616,
  [SMALL_STATE(21)] = 645,
  [SMALL_STATE(22)] = 666,
  [SMALL_STATE(23)] = 687,
  [SMALL_STATE(24)] = 708,
  [SMALL_STATE(25)] = 729,
  [SMALL_STATE(26)] = 758,
  [SMALL_STATE(27)] = 778,
  [SMALL_STATE(28)] = 798,
  [SMALL_STATE(29)] = 818,
  [SMALL_STATE(30)] = 840,
  [SMALL_STATE(31)] = 862,
  [SMALL_STATE(32)] = 882,
  [SMALL_STATE(33)] = 902,
  [SMALL_STATE(34)] = 922,
  [SMALL_STATE(35)] = 942,
  [SMALL_STATE(36)] = 962,
  [SMALL_STATE(37)] = 982,
  [SMALL_STATE(38)] = 1002,
  [SMALL_STATE(39)] = 1022,
  [SMALL_STATE(40)] = 1042,
  [SMALL_STATE(41)] = 1062,
  [SMALL_STATE(42)] = 1082,
  [SMALL_STATE(43)] = 1102,
  [SMALL_STATE(44)] = 1122,
  [SMALL_STATE(45)] = 1142,
  [SMALL_STATE(46)] = 1168,
  [SMALL_STATE(47)] = 1194,
  [SMALL_STATE(48)] = 1220,
  [SMALL_STATE(49)] = 1246,
  [SMALL_STATE(50)] = 1272,
  [SMALL_STATE(51)] = 1298,
  [SMALL_STATE(52)] = 1324,
  [SMALL_STATE(53)] = 1350,
  [SMALL_STATE(54)] = 1376,
  [SMALL_STATE(55)] = 1402,
  [SMALL_STATE(56)] = 1418,
  [SMALL_STATE(57)] = 1434,
  [SMALL_STATE(58)] = 1452,
  [SMALL_STATE(59)] = 1468,
  [SMALL_STATE(60)] = 1484,
  [SMALL_STATE(61)] = 1500,
  [SMALL_STATE(62)] = 1518,
  [SMALL_STATE(63)] = 1534,
  [SMALL_STATE(64)] = 1550,
  [SMALL_STATE(65)] = 1566,
  [SMALL_STATE(66)] = 1585,
  [SMALL_STATE(67)] = 1600,
  [SMALL_STATE(68)] = 1619,
  [SMALL_STATE(69)] = 1638,
  [SMALL_STATE(70)] = 1657,
  [SMALL_STATE(71)] = 1672,
  [SMALL_STATE(72)] = 1691,
  [SMALL_STATE(73)] = 1711,
  [SMALL_STATE(74)] = 1731,
  [SMALL_STATE(75)] = 1751,
  [SMALL_STATE(76)] = 1762,
  [SMALL_STATE(77)] = 1779,
  [SMALL_STATE(78)] = 1796,
  [SMALL_STATE(79)] = 1811,
  [SMALL_STATE(80)] = 1822,
  [SMALL_STATE(81)] = 1839,
  [SMALL_STATE(82)] = 1850,
  [SMALL_STATE(83)] = 1865,
  [SMALL_STATE(84)] = 1877,
  [SMALL_STATE(85)] = 1887,
  [SMALL_STATE(86)] = 1897,
  [SMALL_STATE(87)] = 1906,
  [SMALL_STATE(88)] = 1917,
  [SMALL_STATE(89)] = 1930,
  [SMALL_STATE(90)] = 1943,
  [SMALL_STATE(91)] = 1954,
  [SMALL_STATE(92)] = 1967,
  [SMALL_STATE(93)] = 1978,
  [SMALL_STATE(94)] = 1991,
  [SMALL_STATE(95)] = 2002,
  [SMALL_STATE(96)] = 2012,
  [SMALL_STATE(97)] = 2022,
  [SMALL_STATE(98)] = 2029,
  [SMALL_STATE(99)] = 2036,
  [SMALL_STATE(100)] = 2043,
  [SMALL_STATE(101)] = 2050,
  [SMALL_STATE(102)] = 2057,
  [SMALL_STATE(103)] = 2064,
  [SMALL_STATE(104)] = 2071,
  [SMALL_STATE(105)] = 2078,
  [SMALL_STATE(106)] = 2085,
  [SMALL_STATE(107)] = 2092,
  [SMALL_STATE(108)] = 2099,
  [SMALL_STATE(109)] = 2106,
  [SMALL_STATE(110)] = 2113,
  [SMALL_STATE(111)] = 2120,
  [SMALL_STATE(112)] = 2127,
  [SMALL_STATE(113)] = 2134,
  [SMALL_STATE(114)] = 2141,
  [SMALL_STATE(115)] = 2148,
  [SMALL_STATE(116)] = 2155,
  [SMALL_STATE(117)] = 2162,
  [SMALL_STATE(118)] = 2169,
  [SMALL_STATE(119)] = 2176,
  [SMALL_STATE(120)] = 2183,
  [SMALL_STATE(121)] = 2190,
  [SMALL_STATE(122)] = 2197,
  [SMALL_STATE(123)] = 2204,
  [SMALL_STATE(124)] = 2211,
  [SMALL_STATE(125)] = 2218,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(10),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(114),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(112),
  [13] = {.entry = {.count = 1, .reusable = false}}, SHIFT(110),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(115),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(123),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(75),
  [25] = {.entry = {.count = 1, .reusable = false}}, SHIFT(75),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(95),
  [29] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(5),
  [41] = {.entry = {.count = 1, .reusable = false}}, SHIFT(24),
  [43] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(21),
  [46] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(8),
  [49] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0),
  [51] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(4),
  [54] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(4),
  [57] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(24),
  [60] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [62] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [64] = {.entry = {.count = 1, .reusable = false}}, SHIFT(4),
  [66] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [68] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [70] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [72] = {.entry = {.count = 1, .reusable = false}}, SHIFT(6),
  [74] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [76] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(10),
  [79] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(114),
  [82] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(112),
  [85] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(110),
  [88] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(115),
  [91] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(13),
  [94] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(9),
  [97] = {.entry = {.count = 1, .reusable = true}}, SHIFT(117),
  [99] = {.entry = {.count = 1, .reusable = false}}, SHIFT(66),
  [101] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [103] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [105] = {.entry = {.count = 1, .reusable = true}}, SHIFT(91),
  [107] = {.entry = {.count = 1, .reusable = false}}, SHIFT(91),
  [109] = {.entry = {.count = 1, .reusable = false}}, SHIFT(70),
  [111] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [113] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [115] = {.entry = {.count = 1, .reusable = true}}, SHIFT(121),
  [117] = {.entry = {.count = 1, .reusable = false}}, SHIFT(121),
  [119] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [121] = {.entry = {.count = 1, .reusable = false}}, SHIFT(34),
  [123] = {.entry = {.count = 1, .reusable = false}}, SHIFT(97),
  [125] = {.entry = {.count = 1, .reusable = false}}, SHIFT(116),
  [127] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [129] = {.entry = {.count = 1, .reusable = true}}, SHIFT(89),
  [131] = {.entry = {.count = 1, .reusable = false}}, SHIFT(89),
  [133] = {.entry = {.count = 1, .reusable = true}}, SHIFT(86),
  [135] = {.entry = {.count = 1, .reusable = false}}, SHIFT(86),
  [137] = {.entry = {.count = 1, .reusable = true}}, SHIFT(84),
  [139] = {.entry = {.count = 1, .reusable = false}}, SHIFT(84),
  [141] = {.entry = {.count = 1, .reusable = false}}, SHIFT(31),
  [143] = {.entry = {.count = 1, .reusable = false}}, SHIFT(120),
  [145] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [147] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_if_block_repeat1, 4, 0, 9),
  [149] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [151] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(10),
  [154] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0),
  [156] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(115),
  [159] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(13),
  [162] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [165] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_null, 1, 0, 0),
  [167] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_null, 1, 0, 0),
  [169] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 2, 0, 0),
  [171] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 2, 0, 0),
  [173] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 3, 0, 0),
  [175] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 3, 0, 0),
  [177] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean, 1, 0, 0),
  [179] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean, 1, 0, 0),
  [181] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_if_block_repeat1, 5, 0, 9),
  [183] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 9, 0, 9),
  [185] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 9, 0, 9),
  [187] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 11, 0, 19),
  [189] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 11, 0, 19),
  [191] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 3, 0, 3),
  [193] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 3, 0, 3),
  [195] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object_block, 5, 0, 5),
  [197] = {.entry = {.count = 1, .reusable = false}}, SHIFT(38),
  [199] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object_block, 5, 0, 5),
  [201] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object_block, 4, 0, 5),
  [203] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [205] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object_block, 4, 0, 5),
  [207] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 6, 0, 9),
  [209] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 6, 0, 9),
  [211] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 4, 0, 6),
  [213] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 4, 0, 6),
  [215] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 4, 0, 7),
  [217] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 4, 0, 7),
  [219] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 5, 0, 9),
  [221] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 5, 0, 9),
  [223] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object_block, 5, 0, 10),
  [225] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object_block, 5, 0, 10),
  [227] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 5, 0, 11),
  [229] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 5, 0, 11),
  [231] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 6, 0, 14),
  [233] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 6, 0, 14),
  [235] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object_block, 6, 0, 16),
  [237] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object_block, 6, 0, 16),
  [239] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 10, 0, 19),
  [241] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 10, 0, 19),
  [243] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 7, 0, 19),
  [245] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 7, 0, 19),
  [247] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 8, 0, 9),
  [249] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 8, 0, 9),
  [251] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 9, 0, 14),
  [253] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 9, 0, 14),
  [255] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 10, 0, 9),
  [257] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 10, 0, 9),
  [259] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 10, 0, 14),
  [261] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 10, 0, 14),
  [263] = {.entry = {.count = 1, .reusable = false}}, SHIFT(30),
  [265] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [267] = {.entry = {.count = 1, .reusable = false}}, SHIFT(29),
  [269] = {.entry = {.count = 1, .reusable = false}}, SHIFT(41),
  [271] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [273] = {.entry = {.count = 1, .reusable = false}}, SHIFT(26),
  [275] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [277] = {.entry = {.count = 1, .reusable = false}}, SHIFT(39),
  [279] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [281] = {.entry = {.count = 1, .reusable = false}}, SHIFT(44),
  [283] = {.entry = {.count = 1, .reusable = false}}, SHIFT(43),
  [285] = {.entry = {.count = 1, .reusable = false}}, SHIFT(42),
  [287] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [289] = {.entry = {.count = 1, .reusable = false}}, SHIFT(27),
  [291] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata, 4, 0, 4),
  [293] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_metadata, 4, 0, 4),
  [295] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gather_statement, 3, 0, 2),
  [297] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_gather_statement, 3, 0, 2),
  [299] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_block, 6, 0, 1),
  [301] = {.entry = {.count = 1, .reusable = false}}, SHIFT(62),
  [303] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_block, 6, 0, 1),
  [305] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_block, 6, 0, 13),
  [307] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_block, 6, 0, 13),
  [309] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_block, 5, 0, 1),
  [311] = {.entry = {.count = 1, .reusable = false}}, SHIFT(60),
  [313] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_block, 5, 0, 1),
  [315] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_block, 7, 0, 18),
  [317] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_block, 7, 0, 18),
  [319] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gather_statement, 5, 0, 8),
  [321] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_gather_statement, 5, 0, 8),
  [323] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata, 3, 0, 1),
  [325] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_metadata, 3, 0, 1),
  [327] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_schema_block_repeat1, 2, 0, 0), SHIFT_REPEAT(2),
  [330] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_schema_block_repeat1, 2, 0, 0),
  [332] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_block_repeat1, 2, 0, 0), SHIFT_REPEAT(65),
  [335] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [337] = {.entry = {.count = 1, .reusable = false}}, SHIFT(88),
  [339] = {.entry = {.count = 1, .reusable = true}}, SHIFT(69),
  [341] = {.entry = {.count = 1, .reusable = false}}, SHIFT(57),
  [343] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [345] = {.entry = {.count = 1, .reusable = false}}, SHIFT(93),
  [347] = {.entry = {.count = 1, .reusable = false}}, SHIFT(61),
  [349] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [351] = {.entry = {.count = 1, .reusable = true}}, SHIFT(85),
  [353] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [355] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [357] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [359] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [361] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0), SHIFT_REPEAT(85),
  [364] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0), SHIFT_REPEAT(17),
  [367] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0), SHIFT_REPEAT(118),
  [370] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0),
  [372] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_primitive, 1, 0, 0),
  [374] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [376] = {.entry = {.count = 1, .reusable = true}}, SHIFT(97),
  [378] = {.entry = {.count = 1, .reusable = false}}, SHIFT(99),
  [380] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_if_block_repeat1, 2, 0, 15),
  [382] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_if_block_repeat1, 2, 0, 15), SHIFT_REPEAT(97),
  [385] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_if_block_repeat1, 2, 0, 15),
  [387] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_inline_attributes_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [390] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_inline_attributes_repeat1, 2, 0, 0),
  [392] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_enum, 2, 0, 0),
  [394] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [396] = {.entry = {.count = 1, .reusable = false}}, SHIFT(108),
  [398] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_array_type, 3, 0, 0),
  [400] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [402] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inline_attributes, 2, 0, 0),
  [404] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_modifier, 2, 0, 0),
  [406] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_modifier, 1, 0, 0),
  [408] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inline_attribute, 2, 0, 12),
  [410] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_field, 3, 0, 17),
  [412] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_field, 3, 0, 17),
  [414] = {.entry = {.count = 1, .reusable = false}}, SHIFT(92),
  [416] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_object, 4, 0, 5),
  [418] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_object, 4, 0, 5),
  [420] = {.entry = {.count = 1, .reusable = true}}, SHIFT(83),
  [422] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [424] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_field, 4, 0, 17),
  [426] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_field, 4, 0, 17),
  [428] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [430] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_object, 5, 0, 10),
  [432] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_object, 5, 0, 10),
  [434] = {.entry = {.count = 1, .reusable = false}}, SHIFT(94),
  [436] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_object, 5, 0, 5),
  [438] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_object, 5, 0, 5),
  [440] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_object, 6, 0, 16),
  [442] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_object, 6, 0, 16),
  [444] = {.entry = {.count = 1, .reusable = true}}, SHIFT(122),
  [446] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [448] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [450] = {.entry = {.count = 1, .reusable = true}}, SHIFT(100),
  [452] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [454] = {.entry = {.count = 1, .reusable = true}}, SHIFT(104),
  [456] = {.entry = {.count = 1, .reusable = true}}, SHIFT(98),
  [458] = {.entry = {.count = 1, .reusable = true}}, SHIFT(124),
  [460] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [462] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [464] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [466] = {.entry = {.count = 1, .reusable = true}}, SHIFT(81),
  [468] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [470] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [472] = {.entry = {.count = 1, .reusable = true}}, SHIFT(103),
  [474] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [476] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [478] = {.entry = {.count = 1, .reusable = true}}, SHIFT(125),
  [480] = {.entry = {.count = 1, .reusable = true}}, SHIFT(96),
  [482] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [484] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [486] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [488] = {.entry = {.count = 1, .reusable = true}}, SHIFT(102),
  [490] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [492] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [494] = {.entry = {.count = 1, .reusable = true}}, SHIFT(107),
  [496] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [498] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [500] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [502] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [504] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_runecfg(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .keyword_lex_fn = ts_lex_keywords,
    .keyword_capture_token = sym_identifier,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
