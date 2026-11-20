#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 104
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 74
#define ALIAS_COUNT 0
#define TOKEN_COUNT 45
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 8
#define MAX_ALIAS_SEQUENCE_LENGTH 11
#define PRODUCTION_ID_COUNT 13

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
  anon_sym_else = 28,
  anon_sym_elseif = 29,
  anon_sym_else_DASHif = 30,
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
  aux_sym_inline_attributes_repeat1 = 72,
  aux_sym_array_repeat1 = 73,
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
  [anon_sym_else] = "else",
  [anon_sym_elseif] = "elseif",
  [anon_sym_else_DASHif] = "else-if",
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
  [anon_sym_else] = anon_sym_else,
  [anon_sym_elseif] = anon_sym_elseif,
  [anon_sym_else_DASHif] = anon_sym_else_DASHif,
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
  [anon_sym_else] = {
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
  field_key = 4,
  field_name = 5,
  field_path = 6,
  field_type = 7,
  field_value = 8,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_alias] = "alias",
  [field_attributes] = "attributes",
  [field_condition] = "condition",
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
  [10] = {.index = 15, .length = 3},
  [11] = {.index = 18, .length = 2},
  [12] = {.index = 20, .length = 2},
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
    {field_attributes, 3},
    {field_key, 0},
    {field_value, 2},
  [18] =
    {field_name, 0},
    {field_value, 1},
  [20] =
    {field_name, 0},
    {field_type, 1},
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
  [6] = 6,
  [7] = 4,
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
  [45] = 23,
  [46] = 22,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 20,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 21,
  [59] = 59,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
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
      if (lookahead == ':') ADVANCE(33);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(47);
      if (lookahead != 0 &&
          lookahead != '\t' &&
          lookahead != '\n') ADVANCE(48);
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
  [6] = {.lex_state = 4},
  [7] = {.lex_state = 1},
  [8] = {.lex_state = 1},
  [9] = {.lex_state = 29},
  [10] = {.lex_state = 1},
  [11] = {.lex_state = 29},
  [12] = {.lex_state = 1},
  [13] = {.lex_state = 1},
  [14] = {.lex_state = 1},
  [15] = {.lex_state = 1},
  [16] = {.lex_state = 1},
  [17] = {.lex_state = 29},
  [18] = {.lex_state = 29},
  [19] = {.lex_state = 29},
  [20] = {.lex_state = 1},
  [21] = {.lex_state = 1},
  [22] = {.lex_state = 1},
  [23] = {.lex_state = 1},
  [24] = {.lex_state = 29},
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
  [77] = {.lex_state = 0},
  [78] = {.lex_state = 29},
  [79] = {.lex_state = 5},
  [80] = {.lex_state = 5},
  [81] = {.lex_state = 0},
  [82] = {.lex_state = 0},
  [83] = {.lex_state = 0},
  [84] = {.lex_state = 0},
  [85] = {.lex_state = 0},
  [86] = {.lex_state = 0},
  [87] = {.lex_state = 0},
  [88] = {.lex_state = 29},
  [89] = {.lex_state = 0},
  [90] = {.lex_state = 0},
  [91] = {.lex_state = 29},
  [92] = {.lex_state = 0},
  [93] = {.lex_state = 5},
  [94] = {.lex_state = 0},
  [95] = {.lex_state = 0},
  [96] = {.lex_state = 0},
  [97] = {.lex_state = 0},
  [98] = {.lex_state = 29},
  [99] = {.lex_state = 0},
  [100] = {.lex_state = 0},
  [101] = {.lex_state = 0},
  [102] = {.lex_state = 29},
  [103] = {.lex_state = 0},
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
    [anon_sym_else] = ACTIONS(1),
    [anon_sym_elseif] = ACTIONS(1),
    [anon_sym_else_DASHif] = ACTIONS(1),
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
    [sym_source_file] = STATE(99),
    [sym__item] = STATE(9),
    [sym_metadata] = STATE(9),
    [sym_gather_statement] = STATE(9),
    [sym_schema_block] = STATE(9),
    [sym_object_block] = STATE(9),
    [sym_if_block] = STATE(9),
    [sym_assignment] = STATE(9),
    [aux_sym_source_file_repeat1] = STATE(9),
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
    STATE(60), 4,
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
    ACTIONS(33), 1,
      anon_sym_LBRACK,
    ACTIONS(43), 1,
      anon_sym_RBRACK,
    ACTIONS(47), 1,
      sym_reference,
    ACTIONS(31), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(41), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(3), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(45), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [107] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(52), 1,
      anon_sym_LBRACK,
    ACTIONS(55), 1,
      anon_sym_RBRACK,
    ACTIONS(60), 1,
      sym_reference,
    ACTIONS(49), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(63), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(5), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(57), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [143] = 6,
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
    STATE(84), 4,
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
  [175] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(33), 1,
      anon_sym_LBRACK,
    ACTIONS(66), 1,
      anon_sym_RBRACK,
    ACTIONS(70), 1,
      sym_reference,
    ACTIONS(31), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(41), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(8), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(68), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [211] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(33), 1,
      anon_sym_LBRACK,
    ACTIONS(39), 1,
      sym_reference,
    ACTIONS(72), 1,
      anon_sym_RBRACK,
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
  [247] = 10,
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
    ACTIONS(74), 1,
      ts_builtin_sym_end,
    ACTIONS(76), 1,
      sym__terminator,
    STATE(11), 8,
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
    ACTIONS(78), 1,
      anon_sym_COLON,
    ACTIONS(82), 1,
      anon_sym_LBRACK,
    ACTIONS(84), 1,
      anon_sym_EQ,
    ACTIONS(88), 1,
      sym_reference,
    ACTIONS(80), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(90), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(86), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(76), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [321] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(92), 1,
      ts_builtin_sym_end,
    ACTIONS(94), 1,
      sym_identifier,
    ACTIONS(97), 1,
      anon_sym_AT,
    ACTIONS(100), 1,
      anon_sym_gather,
    ACTIONS(103), 1,
      anon_sym_schema,
    ACTIONS(106), 1,
      anon_sym_if,
    ACTIONS(109), 1,
      sym_string,
    ACTIONS(112), 1,
      sym__terminator,
    STATE(11), 8,
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
    ACTIONS(82), 1,
      anon_sym_LBRACK,
    ACTIONS(117), 1,
      sym_reference,
    ACTIONS(119), 1,
      sym__terminator,
    ACTIONS(80), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(90), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(115), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(101), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [392] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(82), 1,
      anon_sym_LBRACK,
    ACTIONS(84), 1,
      anon_sym_EQ,
    ACTIONS(88), 1,
      sym_reference,
    ACTIONS(80), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(90), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(86), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(76), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [425] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(82), 1,
      anon_sym_LBRACK,
    ACTIONS(123), 1,
      sym_reference,
    ACTIONS(80), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(90), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(121), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(71), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [455] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(82), 1,
      anon_sym_LBRACK,
    ACTIONS(127), 1,
      sym_reference,
    ACTIONS(80), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(90), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(125), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(73), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [485] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(82), 1,
      anon_sym_LBRACK,
    ACTIONS(131), 1,
      sym_reference,
    ACTIONS(80), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(90), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(129), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(68), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [515] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(133), 1,
      anon_sym_end,
    ACTIONS(137), 1,
      sym__terminator,
    ACTIONS(135), 3,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
    STATE(18), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [546] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(139), 1,
      anon_sym_end,
    ACTIONS(143), 1,
      sym__terminator,
    ACTIONS(141), 3,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
    STATE(19), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [577] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(145), 1,
      sym_identifier,
    ACTIONS(150), 1,
      anon_sym_if,
    ACTIONS(153), 1,
      sym_string,
    ACTIONS(156), 1,
      sym__terminator,
    ACTIONS(148), 4,
      anon_sym_end,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
    STATE(19), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [606] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(159), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(161), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [627] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(163), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(165), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [648] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(167), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(169), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [669] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(171), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(173), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [690] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(175), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(177), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [710] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(179), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(181), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [730] = 3,
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
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [750] = 3,
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
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [770] = 3,
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
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [790] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(195), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(197), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [810] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(199), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(201), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [830] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(203), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(205), 8,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [850] = 3,
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
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [870] = 3,
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
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [890] = 3,
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
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [910] = 3,
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
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      sym_identifier,
  [930] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(223), 1,
      anon_sym_end,
    ACTIONS(225), 1,
      sym__terminator,
    STATE(37), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [956] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(143), 1,
      sym__terminator,
    ACTIONS(227), 1,
      anon_sym_end,
    STATE(19), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [982] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(229), 1,
      anon_sym_end,
    ACTIONS(231), 1,
      sym__terminator,
    STATE(43), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1008] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(143), 1,
      sym__terminator,
    ACTIONS(233), 1,
      anon_sym_end,
    STATE(19), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1034] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(235), 1,
      anon_sym_end,
    ACTIONS(237), 1,
      sym__terminator,
    STATE(39), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1060] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(233), 1,
      anon_sym_end,
    ACTIONS(239), 1,
      sym__terminator,
    STATE(42), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1086] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(143), 1,
      sym__terminator,
    ACTIONS(241), 1,
      anon_sym_end,
    STATE(19), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1112] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(17), 1,
      sym_string,
    ACTIONS(143), 1,
      sym__terminator,
    ACTIONS(235), 1,
      anon_sym_end,
    STATE(19), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1138] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(243), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(245), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1154] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(173), 3,
      anon_sym_RBRACK,
      sym_string,
      sym__terminator,
    ACTIONS(171), 5,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      anon_sym_with,
      sym_identifier,
  [1170] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(169), 3,
      anon_sym_RBRACK,
      sym_string,
      sym__terminator,
    ACTIONS(167), 5,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      anon_sym_with,
      sym_identifier,
  [1186] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(247), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(249), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1202] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(251), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(253), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1218] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(255), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(257), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1234] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(259), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(261), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1250] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(263), 4,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym_string,
      sym__terminator,
    ACTIONS(265), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1266] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(267), 1,
      sym_identifier,
    ACTIONS(270), 1,
      anon_sym_end,
    ACTIONS(272), 1,
      sym__terminator,
    STATE(52), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1285] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(275), 1,
      sym_identifier,
    ACTIONS(277), 1,
      anon_sym_end,
    ACTIONS(279), 1,
      sym__terminator,
    STATE(52), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1304] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(161), 2,
      sym_string,
      sym__terminator,
    ACTIONS(159), 5,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      anon_sym_with,
      sym_identifier,
  [1319] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(275), 1,
      sym_identifier,
    ACTIONS(281), 1,
      anon_sym_end,
    ACTIONS(283), 1,
      sym__terminator,
    STATE(56), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1338] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(275), 1,
      sym_identifier,
    ACTIONS(279), 1,
      sym__terminator,
    ACTIONS(285), 1,
      anon_sym_end,
    STATE(52), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1357] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(275), 1,
      sym_identifier,
    ACTIONS(287), 1,
      anon_sym_end,
    ACTIONS(289), 1,
      sym__terminator,
    STATE(53), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1376] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(165), 2,
      sym_string,
      sym__terminator,
    ACTIONS(163), 5,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      anon_sym_with,
      sym_identifier,
  [1391] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(291), 1,
      anon_sym_required,
    ACTIONS(293), 1,
      anon_sym_default,
    ACTIONS(295), 1,
      anon_sym_range,
    ACTIONS(297), 1,
      sym__terminator,
    STATE(61), 2,
      sym_schema_modifier,
      aux_sym_schema_field_repeat1,
  [1411] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(291), 1,
      anon_sym_required,
    ACTIONS(293), 1,
      anon_sym_default,
    ACTIONS(295), 1,
      anon_sym_range,
    ACTIONS(299), 1,
      sym__terminator,
    STATE(59), 2,
      sym_schema_modifier,
      aux_sym_schema_field_repeat1,
  [1431] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(301), 1,
      anon_sym_required,
    ACTIONS(304), 1,
      anon_sym_default,
    ACTIONS(307), 1,
      anon_sym_range,
    ACTIONS(310), 1,
      sym__terminator,
    STATE(61), 2,
      sym_schema_modifier,
      aux_sym_schema_field_repeat1,
  [1451] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(312), 5,
      anon_sym_RBRACK,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1462] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(314), 5,
      anon_sym_RBRACK,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1473] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(318), 1,
      sym__terminator,
    ACTIONS(316), 2,
      sym_identifier,
      sym_string,
    STATE(66), 2,
      sym_inline_attribute,
      aux_sym_inline_attributes_repeat1,
  [1488] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(320), 5,
      anon_sym_RBRACK,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1499] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(325), 1,
      sym__terminator,
    ACTIONS(322), 2,
      sym_identifier,
      sym_string,
    STATE(66), 2,
      sym_inline_attribute,
      aux_sym_inline_attributes_repeat1,
  [1514] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(327), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1524] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(329), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1534] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(316), 2,
      sym_identifier,
      sym_string,
    STATE(64), 2,
      sym_inline_attribute,
      aux_sym_inline_attributes_repeat1,
  [1546] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(333), 1,
      sym__terminator,
    ACTIONS(331), 2,
      anon_sym_end,
      sym_identifier,
  [1557] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(335), 1,
      anon_sym_with,
    ACTIONS(337), 1,
      sym__terminator,
    STATE(96), 1,
      sym_inline_attributes,
  [1570] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(341), 1,
      sym__terminator,
    ACTIONS(339), 2,
      anon_sym_end,
      sym_identifier,
  [1581] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(343), 3,
      sym_identifier,
      sym_string,
      sym__terminator,
  [1590] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(347), 1,
      sym__terminator,
    ACTIONS(345), 2,
      anon_sym_end,
      sym_identifier,
  [1601] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(351), 1,
      sym__terminator,
    ACTIONS(349), 2,
      anon_sym_end,
      sym_identifier,
  [1612] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(335), 1,
      anon_sym_with,
    ACTIONS(353), 1,
      sym__terminator,
    STATE(82), 1,
      sym_inline_attributes,
  [1625] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(82), 1,
      anon_sym_LBRACK,
    STATE(62), 1,
      sym_array,
  [1635] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(355), 1,
      anon_sym_as,
    ACTIONS(357), 1,
      sym__terminator,
  [1645] = 3,
    ACTIONS(359), 1,
      sym_comment,
    ACTIONS(361), 1,
      anon_sym_COLON,
    ACTIONS(363), 1,
      sym_condition,
  [1655] = 3,
    ACTIONS(359), 1,
      sym_comment,
    ACTIONS(365), 1,
      anon_sym_COLON,
    ACTIONS(367), 1,
      sym_condition,
  [1665] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(369), 1,
      anon_sym_COLON,
  [1672] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(371), 1,
      sym__terminator,
  [1679] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(373), 1,
      sym__terminator,
  [1686] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(375), 1,
      anon_sym_RBRACK,
  [1693] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(377), 1,
      anon_sym_COLON,
  [1700] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(379), 1,
      sym_string,
  [1707] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(381), 1,
      anon_sym_COLON,
  [1714] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(129), 1,
      sym_range,
  [1721] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(383), 1,
      sym__terminator,
  [1728] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(385), 1,
      sym__terminator,
  [1735] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(387), 1,
      sym_identifier,
  [1742] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(389), 1,
      sym__terminator,
  [1749] = 2,
    ACTIONS(359), 1,
      sym_comment,
    ACTIONS(391), 1,
      sym_condition,
  [1756] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(393), 1,
      anon_sym_COLON,
  [1763] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(395), 1,
      sym__terminator,
  [1770] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(397), 1,
      sym__terminator,
  [1777] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(399), 1,
      sym__terminator,
  [1784] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(401), 1,
      sym_identifier,
  [1791] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(403), 1,
      ts_builtin_sym_end,
  [1798] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(405), 1,
      sym__terminator,
  [1805] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(407), 1,
      sym__terminator,
  [1812] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(409), 1,
      sym_identifier,
  [1819] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(411), 1,
      sym__terminator,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 35,
  [SMALL_STATE(4)] = 71,
  [SMALL_STATE(5)] = 107,
  [SMALL_STATE(6)] = 143,
  [SMALL_STATE(7)] = 175,
  [SMALL_STATE(8)] = 211,
  [SMALL_STATE(9)] = 247,
  [SMALL_STATE(10)] = 285,
  [SMALL_STATE(11)] = 321,
  [SMALL_STATE(12)] = 359,
  [SMALL_STATE(13)] = 392,
  [SMALL_STATE(14)] = 425,
  [SMALL_STATE(15)] = 455,
  [SMALL_STATE(16)] = 485,
  [SMALL_STATE(17)] = 515,
  [SMALL_STATE(18)] = 546,
  [SMALL_STATE(19)] = 577,
  [SMALL_STATE(20)] = 606,
  [SMALL_STATE(21)] = 627,
  [SMALL_STATE(22)] = 648,
  [SMALL_STATE(23)] = 669,
  [SMALL_STATE(24)] = 690,
  [SMALL_STATE(25)] = 710,
  [SMALL_STATE(26)] = 730,
  [SMALL_STATE(27)] = 750,
  [SMALL_STATE(28)] = 770,
  [SMALL_STATE(29)] = 790,
  [SMALL_STATE(30)] = 810,
  [SMALL_STATE(31)] = 830,
  [SMALL_STATE(32)] = 850,
  [SMALL_STATE(33)] = 870,
  [SMALL_STATE(34)] = 890,
  [SMALL_STATE(35)] = 910,
  [SMALL_STATE(36)] = 930,
  [SMALL_STATE(37)] = 956,
  [SMALL_STATE(38)] = 982,
  [SMALL_STATE(39)] = 1008,
  [SMALL_STATE(40)] = 1034,
  [SMALL_STATE(41)] = 1060,
  [SMALL_STATE(42)] = 1086,
  [SMALL_STATE(43)] = 1112,
  [SMALL_STATE(44)] = 1138,
  [SMALL_STATE(45)] = 1154,
  [SMALL_STATE(46)] = 1170,
  [SMALL_STATE(47)] = 1186,
  [SMALL_STATE(48)] = 1202,
  [SMALL_STATE(49)] = 1218,
  [SMALL_STATE(50)] = 1234,
  [SMALL_STATE(51)] = 1250,
  [SMALL_STATE(52)] = 1266,
  [SMALL_STATE(53)] = 1285,
  [SMALL_STATE(54)] = 1304,
  [SMALL_STATE(55)] = 1319,
  [SMALL_STATE(56)] = 1338,
  [SMALL_STATE(57)] = 1357,
  [SMALL_STATE(58)] = 1376,
  [SMALL_STATE(59)] = 1391,
  [SMALL_STATE(60)] = 1411,
  [SMALL_STATE(61)] = 1431,
  [SMALL_STATE(62)] = 1451,
  [SMALL_STATE(63)] = 1462,
  [SMALL_STATE(64)] = 1473,
  [SMALL_STATE(65)] = 1488,
  [SMALL_STATE(66)] = 1499,
  [SMALL_STATE(67)] = 1514,
  [SMALL_STATE(68)] = 1524,
  [SMALL_STATE(69)] = 1534,
  [SMALL_STATE(70)] = 1546,
  [SMALL_STATE(71)] = 1557,
  [SMALL_STATE(72)] = 1570,
  [SMALL_STATE(73)] = 1581,
  [SMALL_STATE(74)] = 1590,
  [SMALL_STATE(75)] = 1601,
  [SMALL_STATE(76)] = 1612,
  [SMALL_STATE(77)] = 1625,
  [SMALL_STATE(78)] = 1635,
  [SMALL_STATE(79)] = 1645,
  [SMALL_STATE(80)] = 1655,
  [SMALL_STATE(81)] = 1665,
  [SMALL_STATE(82)] = 1672,
  [SMALL_STATE(83)] = 1679,
  [SMALL_STATE(84)] = 1686,
  [SMALL_STATE(85)] = 1693,
  [SMALL_STATE(86)] = 1700,
  [SMALL_STATE(87)] = 1707,
  [SMALL_STATE(88)] = 1714,
  [SMALL_STATE(89)] = 1721,
  [SMALL_STATE(90)] = 1728,
  [SMALL_STATE(91)] = 1735,
  [SMALL_STATE(92)] = 1742,
  [SMALL_STATE(93)] = 1749,
  [SMALL_STATE(94)] = 1756,
  [SMALL_STATE(95)] = 1763,
  [SMALL_STATE(96)] = 1770,
  [SMALL_STATE(97)] = 1777,
  [SMALL_STATE(98)] = 1784,
  [SMALL_STATE(99)] = 1791,
  [SMALL_STATE(100)] = 1798,
  [SMALL_STATE(101)] = 1805,
  [SMALL_STATE(102)] = 1812,
  [SMALL_STATE(103)] = 1819,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(10),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(91),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(86),
  [13] = {.entry = {.count = 1, .reusable = false}}, SHIFT(98),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(93),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(95),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [25] = {.entry = {.count = 1, .reusable = false}}, SHIFT(63),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [29] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(5),
  [41] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [45] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(3),
  [49] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [52] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(7),
  [55] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0),
  [57] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(5),
  [60] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(5),
  [63] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(21),
  [66] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [68] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [70] = {.entry = {.count = 1, .reusable = false}}, SHIFT(8),
  [72] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [74] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [76] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [78] = {.entry = {.count = 1, .reusable = true}}, SHIFT(103),
  [80] = {.entry = {.count = 1, .reusable = false}}, SHIFT(54),
  [82] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [84] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [86] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
  [88] = {.entry = {.count = 1, .reusable = false}}, SHIFT(76),
  [90] = {.entry = {.count = 1, .reusable = false}}, SHIFT(58),
  [92] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [94] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(10),
  [97] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(91),
  [100] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(86),
  [103] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(98),
  [106] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(93),
  [109] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(13),
  [112] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(11),
  [115] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [117] = {.entry = {.count = 1, .reusable = false}}, SHIFT(101),
  [119] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [121] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [123] = {.entry = {.count = 1, .reusable = false}}, SHIFT(71),
  [125] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [127] = {.entry = {.count = 1, .reusable = false}}, SHIFT(73),
  [129] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [131] = {.entry = {.count = 1, .reusable = false}}, SHIFT(68),
  [133] = {.entry = {.count = 1, .reusable = false}}, SHIFT(25),
  [135] = {.entry = {.count = 1, .reusable = false}}, SHIFT(80),
  [137] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [139] = {.entry = {.count = 1, .reusable = false}}, SHIFT(32),
  [141] = {.entry = {.count = 1, .reusable = false}}, SHIFT(79),
  [143] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [145] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(10),
  [148] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0),
  [150] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(93),
  [153] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(13),
  [156] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(19),
  [159] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_null, 1, 0, 0),
  [161] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_null, 1, 0, 0),
  [163] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean, 1, 0, 0),
  [165] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean, 1, 0, 0),
  [167] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 2, 0, 0),
  [169] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 2, 0, 0),
  [171] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 3, 0, 0),
  [173] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 3, 0, 0),
  [175] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 10, 0, 9),
  [177] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 10, 0, 9),
  [179] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 5, 0, 9),
  [181] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 5, 0, 9),
  [183] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 4, 0, 6),
  [185] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 4, 0, 6),
  [187] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 4, 0, 7),
  [189] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 4, 0, 7),
  [191] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 3, 0, 3),
  [193] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 3, 0, 3),
  [195] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object_block, 4, 0, 5),
  [197] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object_block, 4, 0, 5),
  [199] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object_block, 5, 0, 5),
  [201] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object_block, 5, 0, 5),
  [203] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 5, 0, 10),
  [205] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 5, 0, 10),
  [207] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 6, 0, 9),
  [209] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 6, 0, 9),
  [211] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 9, 0, 9),
  [213] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 9, 0, 9),
  [215] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 8, 0, 9),
  [217] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 8, 0, 9),
  [219] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 11, 0, 9),
  [221] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 11, 0, 9),
  [223] = {.entry = {.count = 1, .reusable = false}}, SHIFT(29),
  [225] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [227] = {.entry = {.count = 1, .reusable = false}}, SHIFT(30),
  [229] = {.entry = {.count = 1, .reusable = false}}, SHIFT(34),
  [231] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [233] = {.entry = {.count = 1, .reusable = false}}, SHIFT(24),
  [235] = {.entry = {.count = 1, .reusable = false}}, SHIFT(33),
  [237] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [239] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [241] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [243] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata, 4, 0, 4),
  [245] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_metadata, 4, 0, 4),
  [247] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gather_statement, 5, 0, 8),
  [249] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_gather_statement, 5, 0, 8),
  [251] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_block, 5, 0, 1),
  [253] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_block, 5, 0, 1),
  [255] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gather_statement, 3, 0, 2),
  [257] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_gather_statement, 3, 0, 2),
  [259] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_block, 6, 0, 1),
  [261] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_block, 6, 0, 1),
  [263] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata, 3, 0, 1),
  [265] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_metadata, 3, 0, 1),
  [267] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_schema_block_repeat1, 2, 0, 0), SHIFT_REPEAT(2),
  [270] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_schema_block_repeat1, 2, 0, 0),
  [272] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_block_repeat1, 2, 0, 0), SHIFT_REPEAT(52),
  [275] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [277] = {.entry = {.count = 1, .reusable = false}}, SHIFT(50),
  [279] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [281] = {.entry = {.count = 1, .reusable = false}}, SHIFT(70),
  [283] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [285] = {.entry = {.count = 1, .reusable = false}}, SHIFT(75),
  [287] = {.entry = {.count = 1, .reusable = false}}, SHIFT(48),
  [289] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [291] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [293] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [295] = {.entry = {.count = 1, .reusable = true}}, SHIFT(88),
  [297] = {.entry = {.count = 1, .reusable = true}}, SHIFT(74),
  [299] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [301] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0), SHIFT_REPEAT(67),
  [304] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [307] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0), SHIFT_REPEAT(88),
  [310] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0),
  [312] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_enum, 2, 0, 0),
  [314] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_primitive, 1, 0, 0),
  [316] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [318] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inline_attributes, 2, 0, 0),
  [320] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_array_type, 3, 0, 0),
  [322] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_inline_attributes_repeat1, 2, 0, 0), SHIFT_REPEAT(15),
  [325] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_inline_attributes_repeat1, 2, 0, 0),
  [327] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_modifier, 1, 0, 0),
  [329] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_modifier, 2, 0, 0),
  [331] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_object, 4, 0, 5),
  [333] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_object, 4, 0, 5),
  [335] = {.entry = {.count = 1, .reusable = true}}, SHIFT(69),
  [337] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [339] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_field, 3, 0, 12),
  [341] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_field, 3, 0, 12),
  [343] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inline_attribute, 2, 0, 11),
  [345] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_field, 4, 0, 12),
  [347] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_field, 4, 0, 12),
  [349] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_object, 5, 0, 5),
  [351] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_object, 5, 0, 5),
  [353] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [355] = {.entry = {.count = 1, .reusable = true}}, SHIFT(102),
  [357] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [359] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [361] = {.entry = {.count = 1, .reusable = false}}, SHIFT(97),
  [363] = {.entry = {.count = 1, .reusable = true}}, SHIFT(81),
  [365] = {.entry = {.count = 1, .reusable = false}}, SHIFT(83),
  [367] = {.entry = {.count = 1, .reusable = true}}, SHIFT(85),
  [369] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [371] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [373] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [375] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [377] = {.entry = {.count = 1, .reusable = true}}, SHIFT(97),
  [379] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [381] = {.entry = {.count = 1, .reusable = true}}, SHIFT(100),
  [383] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [385] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [387] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [389] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [391] = {.entry = {.count = 1, .reusable = true}}, SHIFT(94),
  [393] = {.entry = {.count = 1, .reusable = true}}, SHIFT(92),
  [395] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [397] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [399] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [401] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [403] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [405] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [407] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [409] = {.entry = {.count = 1, .reusable = true}}, SHIFT(89),
  [411] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
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
