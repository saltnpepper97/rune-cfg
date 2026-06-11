#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 131
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 71
#define ALIAS_COUNT 0
#define TOKEN_COUNT 45
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 7
#define MAX_ALIAS_SEQUENCE_LENGTH 11
#define PRODUCTION_ID_COUNT 10

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
  anon_sym_endif = 31,
  anon_sym_EQ = 32,
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
  sym__value = 62,
  sym_array = 63,
  sym_boolean = 64,
  sym_null = 65,
  aux_sym_source_file_repeat1 = 66,
  aux_sym_schema_block_repeat1 = 67,
  aux_sym_schema_field_repeat1 = 68,
  aux_sym_object_block_repeat1 = 69,
  aux_sym_array_repeat1 = 70,
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
  [anon_sym_endif] = "endif",
  [anon_sym_EQ] = "=",
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
  [sym__value] = "_value",
  [sym_array] = "array",
  [sym_boolean] = "boolean",
  [sym_null] = "null",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_schema_block_repeat1] = "schema_block_repeat1",
  [aux_sym_schema_field_repeat1] = "schema_field_repeat1",
  [aux_sym_object_block_repeat1] = "object_block_repeat1",
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
  [anon_sym_endif] = anon_sym_endif,
  [anon_sym_EQ] = anon_sym_EQ,
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
  [sym__value] = sym__value,
  [sym_array] = sym_array,
  [sym_boolean] = sym_boolean,
  [sym_null] = sym_null,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_schema_block_repeat1] = aux_sym_schema_block_repeat1,
  [aux_sym_schema_field_repeat1] = aux_sym_schema_field_repeat1,
  [aux_sym_object_block_repeat1] = aux_sym_object_block_repeat1,
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
  [anon_sym_endif] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
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
  [aux_sym_array_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_alias = 1,
  field_condition = 2,
  field_key = 3,
  field_name = 4,
  field_path = 5,
  field_type = 6,
  field_value = 7,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_alias] = "alias",
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
  [7] = {.index = 9, .length = 2},
  [8] = {.index = 11, .length = 1},
  [9] = {.index = 12, .length = 2},
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
    {field_alias, 3},
    {field_path, 1},
  [11] =
    {field_condition, 1},
  [12] =
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
  [6] = 4,
  [7] = 7,
  [8] = 3,
  [9] = 9,
  [10] = 9,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 14,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 21,
  [24] = 22,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 25,
  [31] = 26,
  [32] = 32,
  [33] = 33,
  [34] = 27,
  [35] = 35,
  [36] = 33,
  [37] = 37,
  [38] = 37,
  [39] = 29,
  [40] = 28,
  [41] = 32,
  [42] = 35,
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
  [53] = 46,
  [54] = 48,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 58,
  [59] = 43,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 44,
  [64] = 45,
  [65] = 49,
  [66] = 52,
  [67] = 47,
  [68] = 50,
  [69] = 51,
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
  [81] = 20,
  [82] = 82,
  [83] = 19,
  [84] = 18,
  [85] = 17,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 91,
  [94] = 94,
  [95] = 92,
  [96] = 94,
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
  [108] = 103,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 113,
  [114] = 114,
  [115] = 115,
  [116] = 116,
  [117] = 117,
  [118] = 113,
  [119] = 119,
  [120] = 117,
  [121] = 114,
  [122] = 115,
  [123] = 100,
  [124] = 101,
  [125] = 125,
  [126] = 125,
  [127] = 127,
  [128] = 116,
  [129] = 98,
  [130] = 106,
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
      if (lookahead == '\n') ADVANCE(100);
      if (lookahead == '#') ADVANCE(31);
      if (lookahead == '@') ADVANCE(32);
      if (lookahead == ']') ADVANCE(37);
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(23);
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
      );
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0);
      END_STATE();
    case 1:
      if (lookahead == 'n') ADVANCE(12);
      if (lookahead == 's') ADVANCE(13);
      END_STATE();
    case 2:
      if (lookahead == 'o') ADVANCE(14);
      END_STATE();
    case 3:
      if (lookahead == 'e') ADVANCE(15);
      END_STATE();
    case 4:
      if (lookahead == 'l') ADVANCE(16);
      if (lookahead == 'n') ADVANCE(17);
      END_STATE();
    case 5:
      if (lookahead == 'l') ADVANCE(18);
      END_STATE();
    case 6:
      if (lookahead == 'a') ADVANCE(19);
      END_STATE();
    case 7:
      if (lookahead == 'f') ADVANCE(20);
      if (lookahead == 'n') ADVANCE(21);
      END_STATE();
    case 8:
      if (lookahead == 'u') ADVANCE(22);
      END_STATE();
    case 9:
      if (lookahead == 'b') ADVANCE(23);
      END_STATE();
    case 10:
      if (lookahead == 'a') ADVANCE(24);
      if (lookahead == 'e') ADVANCE(25);
      END_STATE();
    case 11:
      if (lookahead == 'c') ADVANCE(26);
      if (lookahead == 't') ADVANCE(27);
      END_STATE();
    case 12:
      if (lookahead == 'y') ADVANCE(28);
      END_STATE();
    case 13:
      ACCEPT_TOKEN(anon_sym_as);
      END_STATE();
    case 14:
      if (lookahead == 'o') ADVANCE(29);
      END_STATE();
    case 15:
      if (lookahead == 'f') ADVANCE(30);
      END_STATE();
    case 16:
      if (lookahead == 's') ADVANCE(31);
      END_STATE();
    case 17:
      if (lookahead == 'd') ADVANCE(32);
      if (lookahead == 'u') ADVANCE(33);
      END_STATE();
    case 18:
      if (lookahead == 'o') ADVANCE(34);
      END_STATE();
    case 19:
      if (lookahead == 't') ADVANCE(35);
      END_STATE();
    case 20:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 21:
      if (lookahead == 't') ADVANCE(36);
      END_STATE();
    case 22:
      if (lookahead == 'm') ADVANCE(37);
      END_STATE();
    case 23:
      if (lookahead == 'j') ADVANCE(38);
      END_STATE();
    case 24:
      if (lookahead == 'n') ADVANCE(39);
      END_STATE();
    case 25:
      if (lookahead == 'g') ADVANCE(40);
      if (lookahead == 'q') ADVANCE(41);
      END_STATE();
    case 26:
      if (lookahead == 'h') ADVANCE(42);
      END_STATE();
    case 27:
      if (lookahead == 'r') ADVANCE(43);
      END_STATE();
    case 28:
      ACCEPT_TOKEN(anon_sym_any);
      END_STATE();
    case 29:
      if (lookahead == 'l') ADVANCE(44);
      END_STATE();
    case 30:
      if (lookahead == 'a') ADVANCE(45);
      END_STATE();
    case 31:
      if (lookahead == 'e') ADVANCE(46);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_end);
      if (lookahead == 'i') ADVANCE(47);
      END_STATE();
    case 33:
      if (lookahead == 'm') ADVANCE(48);
      END_STATE();
    case 34:
      if (lookahead == 'a') ADVANCE(49);
      END_STATE();
    case 35:
      if (lookahead == 'h') ADVANCE(50);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_int);
      if (lookahead == 'e') ADVANCE(51);
      END_STATE();
    case 37:
      if (lookahead == 'b') ADVANCE(52);
      END_STATE();
    case 38:
      if (lookahead == 'e') ADVANCE(53);
      END_STATE();
    case 39:
      if (lookahead == 'g') ADVANCE(54);
      END_STATE();
    case 40:
      if (lookahead == 'e') ADVANCE(55);
      END_STATE();
    case 41:
      if (lookahead == 'u') ADVANCE(56);
      END_STATE();
    case 42:
      if (lookahead == 'e') ADVANCE(57);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(anon_sym_str);
      if (lookahead == 'i') ADVANCE(58);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(anon_sym_bool);
      if (lookahead == 'e') ADVANCE(59);
      END_STATE();
    case 45:
      if (lookahead == 'u') ADVANCE(60);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(anon_sym_else);
      if (lookahead == '-') ADVANCE(61);
      if (lookahead == 'i') ADVANCE(62);
      END_STATE();
    case 47:
      if (lookahead == 'f') ADVANCE(63);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(anon_sym_enum);
      END_STATE();
    case 49:
      if (lookahead == 't') ADVANCE(64);
      END_STATE();
    case 50:
      if (lookahead == 'e') ADVANCE(65);
      END_STATE();
    case 51:
      if (lookahead == 'g') ADVANCE(66);
      END_STATE();
    case 52:
      if (lookahead == 'e') ADVANCE(67);
      END_STATE();
    case 53:
      if (lookahead == 'c') ADVANCE(68);
      END_STATE();
    case 54:
      if (lookahead == 'e') ADVANCE(69);
      END_STATE();
    case 55:
      if (lookahead == 'x') ADVANCE(70);
      END_STATE();
    case 56:
      if (lookahead == 'i') ADVANCE(71);
      END_STATE();
    case 57:
      if (lookahead == 'm') ADVANCE(72);
      END_STATE();
    case 58:
      if (lookahead == 'n') ADVANCE(73);
      END_STATE();
    case 59:
      if (lookahead == 'a') ADVANCE(74);
      END_STATE();
    case 60:
      if (lookahead == 'l') ADVANCE(75);
      END_STATE();
    case 61:
      if (lookahead == 'i') ADVANCE(76);
      END_STATE();
    case 62:
      if (lookahead == 'f') ADVANCE(77);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(anon_sym_endif);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(anon_sym_float);
      END_STATE();
    case 65:
      if (lookahead == 'r') ADVANCE(78);
      END_STATE();
    case 66:
      if (lookahead == 'e') ADVANCE(79);
      END_STATE();
    case 67:
      if (lookahead == 'r') ADVANCE(80);
      END_STATE();
    case 68:
      if (lookahead == 't') ADVANCE(81);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(anon_sym_range);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(anon_sym_regex);
      END_STATE();
    case 71:
      if (lookahead == 'r') ADVANCE(82);
      END_STATE();
    case 72:
      if (lookahead == 'a') ADVANCE(83);
      END_STATE();
    case 73:
      if (lookahead == 'g') ADVANCE(84);
      END_STATE();
    case 74:
      if (lookahead == 'n') ADVANCE(85);
      END_STATE();
    case 75:
      if (lookahead == 't') ADVANCE(86);
      END_STATE();
    case 76:
      if (lookahead == 'f') ADVANCE(87);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(anon_sym_elseif);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(anon_sym_gather);
      END_STATE();
    case 79:
      if (lookahead == 'r') ADVANCE(88);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(anon_sym_number);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(anon_sym_object);
      END_STATE();
    case 82:
      if (lookahead == 'e') ADVANCE(89);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(anon_sym_schema);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(anon_sym_string);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(anon_sym_boolean);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(anon_sym_default);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(anon_sym_else_DASHif);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(anon_sym_integer);
      END_STATE();
    case 89:
      if (lookahead == 'd') ADVANCE(90);
      END_STATE();
    case 90:
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
  [9] = {.lex_state = 1},
  [10] = {.lex_state = 1},
  [11] = {.lex_state = 1},
  [12] = {.lex_state = 29},
  [13] = {.lex_state = 29},
  [14] = {.lex_state = 1},
  [15] = {.lex_state = 1},
  [16] = {.lex_state = 1},
  [17] = {.lex_state = 1},
  [18] = {.lex_state = 1},
  [19] = {.lex_state = 1},
  [20] = {.lex_state = 1},
  [21] = {.lex_state = 29},
  [22] = {.lex_state = 29},
  [23] = {.lex_state = 29},
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
  [91] = {.lex_state = 5},
  [92] = {.lex_state = 5},
  [93] = {.lex_state = 5},
  [94] = {.lex_state = 0},
  [95] = {.lex_state = 5},
  [96] = {.lex_state = 0},
  [97] = {.lex_state = 0},
  [98] = {.lex_state = 5},
  [99] = {.lex_state = 0},
  [100] = {.lex_state = 0},
  [101] = {.lex_state = 0},
  [102] = {.lex_state = 0},
  [103] = {.lex_state = 0},
  [104] = {.lex_state = 0},
  [105] = {.lex_state = 0},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 29},
  [108] = {.lex_state = 0},
  [109] = {.lex_state = 29},
  [110] = {.lex_state = 0},
  [111] = {.lex_state = 0},
  [112] = {.lex_state = 29},
  [113] = {.lex_state = 0},
  [114] = {.lex_state = 0},
  [115] = {.lex_state = 0},
  [116] = {.lex_state = 0},
  [117] = {.lex_state = 0},
  [118] = {.lex_state = 0},
  [119] = {.lex_state = 0},
  [120] = {.lex_state = 0},
  [121] = {.lex_state = 0},
  [122] = {.lex_state = 0},
  [123] = {.lex_state = 0},
  [124] = {.lex_state = 0},
  [125] = {.lex_state = 0},
  [126] = {.lex_state = 0},
  [127] = {.lex_state = 29},
  [128] = {.lex_state = 0},
  [129] = {.lex_state = 5},
  [130] = {.lex_state = 0},
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
    [anon_sym_endif] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
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
    [sym__item] = STATE(12),
    [sym_metadata] = STATE(12),
    [sym_gather_statement] = STATE(12),
    [sym_schema_block] = STATE(12),
    [sym_object_block] = STATE(12),
    [sym_if_block] = STATE(12),
    [sym_assignment] = STATE(12),
    [aux_sym_source_file_repeat1] = STATE(12),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_identifier] = ACTIONS(7),
    [sym_comment] = ACTIONS(3),
    [anon_sym_AT] = ACTIONS(9),
    [anon_sym_gather] = ACTIONS(11),
    [anon_sym_schema] = ACTIONS(13),
    [anon_sym_if] = ACTIONS(15),
    [sym__terminator] = ACTIONS(17),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(19), 1,
      anon_sym_COLON,
    ACTIONS(25), 1,
      anon_sym_enum,
    ACTIONS(27), 1,
      anon_sym_LBRACK,
    ACTIONS(23), 3,
      anon_sym_str,
      anon_sym_int,
      anon_sym_bool,
    STATE(74), 4,
      sym__schema_type,
      sym_schema_primitive,
      sym_schema_enum,
      sym_schema_array_type,
    ACTIONS(21), 9,
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
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    ACTIONS(33), 1,
      anon_sym_RBRACK,
    ACTIONS(37), 1,
      sym_reference,
    ACTIONS(29), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(39), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(5), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(35), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [71] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    ACTIONS(41), 1,
      anon_sym_RBRACK,
    ACTIONS(45), 1,
      sym_reference,
    ACTIONS(29), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(39), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(3), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(43), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [107] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(50), 1,
      anon_sym_LBRACK,
    ACTIONS(53), 1,
      anon_sym_RBRACK,
    ACTIONS(58), 1,
      sym_reference,
    ACTIONS(47), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(61), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(5), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(55), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [143] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    ACTIONS(64), 1,
      anon_sym_RBRACK,
    ACTIONS(68), 1,
      sym_reference,
    ACTIONS(29), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(39), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(8), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(66), 6,
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
      anon_sym_LBRACK,
    ACTIONS(70), 1,
      anon_sym_enum,
    ACTIONS(23), 3,
      anon_sym_str,
      anon_sym_int,
      anon_sym_bool,
    STATE(102), 4,
      sym__schema_type,
      sym_schema_primitive,
      sym_schema_enum,
      sym_schema_array_type,
    ACTIONS(21), 9,
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
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    ACTIONS(37), 1,
      sym_reference,
    ACTIONS(72), 1,
      anon_sym_RBRACK,
    ACTIONS(29), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(39), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(5), 5,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
      aux_sym_array_repeat1,
    ACTIONS(35), 6,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [247] = 9,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    ACTIONS(74), 1,
      anon_sym_COLON,
    ACTIONS(76), 1,
      anon_sym_EQ,
    ACTIONS(80), 1,
      sym_reference,
    ACTIONS(29), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(39), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(78), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(108), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [283] = 9,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    ACTIONS(82), 1,
      anon_sym_COLON,
    ACTIONS(84), 1,
      anon_sym_EQ,
    ACTIONS(88), 1,
      sym_reference,
    ACTIONS(29), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(39), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(86), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(103), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [319] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    ACTIONS(92), 1,
      sym_reference,
    ACTIONS(94), 1,
      sym__terminator,
    ACTIONS(29), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(39), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(90), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(105), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [352] = 9,
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
    ACTIONS(96), 1,
      ts_builtin_sym_end,
    ACTIONS(98), 1,
      sym__terminator,
    STATE(13), 8,
      sym__item,
      sym_metadata,
      sym_gather_statement,
      sym_schema_block,
      sym_object_block,
      sym_if_block,
      sym_assignment,
      aux_sym_source_file_repeat1,
  [387] = 9,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(100), 1,
      ts_builtin_sym_end,
    ACTIONS(102), 1,
      sym_identifier,
    ACTIONS(105), 1,
      anon_sym_AT,
    ACTIONS(108), 1,
      anon_sym_gather,
    ACTIONS(111), 1,
      anon_sym_schema,
    ACTIONS(114), 1,
      anon_sym_if,
    ACTIONS(117), 1,
      sym__terminator,
    STATE(13), 8,
      sym__item,
      sym_metadata,
      sym_gather_statement,
      sym_schema_block,
      sym_object_block,
      sym_if_block,
      sym_assignment,
      aux_sym_source_file_repeat1,
  [422] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    ACTIONS(122), 1,
      sym_reference,
    ACTIONS(29), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(39), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(120), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(130), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [452] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    ACTIONS(126), 1,
      sym_reference,
    ACTIONS(29), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(39), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(124), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(106), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [482] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(130), 1,
      anon_sym_LBRACK,
    ACTIONS(134), 1,
      sym_reference,
    ACTIONS(128), 2,
      anon_sym_null,
      anon_sym_None,
    ACTIONS(136), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(132), 4,
      sym_conditional_value,
      sym_string,
      sym_regex,
      sym_number,
    STATE(82), 4,
      sym__value,
      sym_array,
      sym_boolean,
      sym_null,
  [512] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(138), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(140), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [533] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(142), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(144), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [554] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(146), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(148), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [575] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(150), 5,
      anon_sym_null,
      sym_reference,
      anon_sym_true,
      anon_sym_false,
      anon_sym_None,
    ACTIONS(152), 8,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
      sym_conditional_value,
      anon_sym_COMMA,
      sym_string,
      sym_regex,
      sym_number,
      sym__terminator,
  [596] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(160), 1,
      anon_sym_endif,
    ACTIONS(162), 1,
      sym__terminator,
    ACTIONS(158), 3,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
    STATE(24), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [624] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(166), 1,
      anon_sym_endif,
    ACTIONS(168), 1,
      sym__terminator,
    ACTIONS(164), 3,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [652] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(172), 1,
      anon_sym_endif,
    ACTIONS(174), 1,
      sym__terminator,
    ACTIONS(170), 3,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
    STATE(22), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [680] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(168), 1,
      sym__terminator,
    ACTIONS(178), 1,
      anon_sym_endif,
    ACTIONS(176), 3,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [708] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(180), 1,
      sym_identifier,
    ACTIONS(183), 1,
      anon_sym_if,
    ACTIONS(188), 1,
      sym__terminator,
    ACTIONS(186), 4,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [734] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(191), 1,
      anon_sym_endif,
    ACTIONS(193), 1,
      sym__terminator,
    STATE(40), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [757] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(195), 1,
      anon_sym_endif,
    ACTIONS(197), 1,
      sym__terminator,
    STATE(32), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [780] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(168), 1,
      sym__terminator,
    ACTIONS(195), 1,
      anon_sym_endif,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [803] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(199), 1,
      anon_sym_end,
    ACTIONS(201), 1,
      sym__terminator,
    STATE(30), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [826] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(186), 1,
      anon_sym_end,
    ACTIONS(203), 1,
      sym_identifier,
    ACTIONS(206), 1,
      anon_sym_if,
    ACTIONS(209), 1,
      sym__terminator,
    STATE(30), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [849] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(212), 1,
      anon_sym_endif,
    ACTIONS(214), 1,
      sym__terminator,
    STATE(28), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [872] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(168), 1,
      sym__terminator,
    ACTIONS(216), 1,
      anon_sym_endif,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [895] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(216), 1,
      anon_sym_endif,
    ACTIONS(218), 1,
      sym__terminator,
    STATE(35), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [918] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(220), 1,
      anon_sym_endif,
    ACTIONS(222), 1,
      sym__terminator,
    STATE(41), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [941] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(168), 1,
      sym__terminator,
    ACTIONS(224), 1,
      anon_sym_endif,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [964] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(226), 1,
      anon_sym_endif,
    ACTIONS(228), 1,
      sym__terminator,
    STATE(42), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [987] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(230), 1,
      anon_sym_end,
    ACTIONS(232), 1,
      sym__terminator,
    STATE(29), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1010] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(234), 1,
      anon_sym_end,
    ACTIONS(236), 1,
      sym__terminator,
    STATE(39), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1033] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_identifier,
    ACTIONS(15), 1,
      anon_sym_if,
    ACTIONS(201), 1,
      sym__terminator,
    ACTIONS(238), 1,
      anon_sym_end,
    STATE(30), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1056] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(168), 1,
      sym__terminator,
    ACTIONS(220), 1,
      anon_sym_endif,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1079] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(168), 1,
      sym__terminator,
    ACTIONS(226), 1,
      anon_sym_endif,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1102] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 1,
      sym_identifier,
    ACTIONS(156), 1,
      anon_sym_if,
    ACTIONS(168), 1,
      sym__terminator,
    ACTIONS(240), 1,
      anon_sym_endif,
    STATE(25), 5,
      sym_object_block,
      sym__object_item,
      sym_if_block,
      sym_assignment,
      aux_sym_object_block_repeat1,
  [1125] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(242), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(244), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1141] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(246), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(248), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1157] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(250), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(252), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1173] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(254), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(256), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1189] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(258), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(260), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1205] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(262), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(264), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1221] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(266), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(268), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1237] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(270), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(272), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1253] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(274), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(276), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1269] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(278), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(280), 5,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_end,
      anon_sym_if,
      sym_identifier,
  [1285] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(254), 1,
      sym__terminator,
    ACTIONS(256), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1300] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(262), 1,
      sym__terminator,
    ACTIONS(264), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1315] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(282), 1,
      sym_identifier,
    ACTIONS(284), 1,
      anon_sym_end,
    ACTIONS(286), 1,
      sym__terminator,
    STATE(72), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1334] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(282), 1,
      sym_identifier,
    ACTIONS(288), 1,
      anon_sym_end,
    ACTIONS(290), 1,
      sym__terminator,
    STATE(73), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1353] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(292), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(294), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1368] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(296), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(298), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1383] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(242), 1,
      sym__terminator,
    ACTIONS(244), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1398] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(300), 1,
      sym_identifier,
    ACTIONS(303), 1,
      anon_sym_end,
    ACTIONS(305), 1,
      sym__terminator,
    STATE(60), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1417] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(308), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(310), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1432] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(312), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(314), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1447] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(246), 1,
      sym__terminator,
    ACTIONS(248), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1462] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(250), 1,
      sym__terminator,
    ACTIONS(252), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1477] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(266), 1,
      sym__terminator,
    ACTIONS(268), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1492] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(278), 1,
      sym__terminator,
    ACTIONS(280), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1507] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(258), 1,
      sym__terminator,
    ACTIONS(260), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1522] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(270), 1,
      sym__terminator,
    ACTIONS(272), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1537] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(274), 1,
      sym__terminator,
    ACTIONS(276), 6,
      anon_sym_if,
      anon_sym_else,
      anon_sym_elseif,
      anon_sym_else_DASHif,
      anon_sym_endif,
      sym_identifier,
  [1552] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(316), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(318), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1567] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(320), 3,
      ts_builtin_sym_end,
      anon_sym_AT,
      sym__terminator,
    ACTIONS(322), 4,
      anon_sym_gather,
      anon_sym_schema,
      anon_sym_if,
      sym_identifier,
  [1582] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(282), 1,
      sym_identifier,
    ACTIONS(324), 1,
      anon_sym_end,
    ACTIONS(326), 1,
      sym__terminator,
    STATE(60), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1601] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(282), 1,
      sym_identifier,
    ACTIONS(326), 1,
      sym__terminator,
    ACTIONS(328), 1,
      anon_sym_end,
    STATE(60), 4,
      sym__schema_item,
      sym_schema_object,
      sym_schema_field,
      aux_sym_schema_block_repeat1,
  [1620] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(330), 1,
      anon_sym_required,
    ACTIONS(332), 1,
      anon_sym_default,
    ACTIONS(334), 1,
      anon_sym_range,
    ACTIONS(336), 1,
      sym__terminator,
    STATE(76), 2,
      sym_schema_modifier,
      aux_sym_schema_field_repeat1,
  [1640] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(338), 1,
      anon_sym_required,
    ACTIONS(341), 1,
      anon_sym_default,
    ACTIONS(344), 1,
      anon_sym_range,
    ACTIONS(347), 1,
      sym__terminator,
    STATE(75), 2,
      sym_schema_modifier,
      aux_sym_schema_field_repeat1,
  [1660] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(330), 1,
      anon_sym_required,
    ACTIONS(332), 1,
      anon_sym_default,
    ACTIONS(334), 1,
      anon_sym_range,
    ACTIONS(349), 1,
      sym__terminator,
    STATE(75), 2,
      sym_schema_modifier,
      aux_sym_schema_field_repeat1,
  [1680] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(351), 5,
      anon_sym_RBRACK,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1691] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(353), 5,
      anon_sym_RBRACK,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1702] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(355), 5,
      anon_sym_RBRACK,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1713] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(357), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1723] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(152), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1733] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(359), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1743] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(148), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1753] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(144), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1763] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(140), 4,
      anon_sym_required,
      anon_sym_default,
      anon_sym_range,
      sym__terminator,
  [1773] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(363), 1,
      sym__terminator,
    ACTIONS(361), 2,
      anon_sym_end,
      sym_identifier,
  [1784] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(367), 1,
      sym__terminator,
    ACTIONS(365), 2,
      anon_sym_end,
      sym_identifier,
  [1795] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(371), 1,
      sym__terminator,
    ACTIONS(369), 2,
      anon_sym_end,
      sym_identifier,
  [1806] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(375), 1,
      sym__terminator,
    ACTIONS(373), 2,
      anon_sym_end,
      sym_identifier,
  [1817] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(377), 1,
      anon_sym_as,
    ACTIONS(379), 1,
      sym__terminator,
  [1827] = 3,
    ACTIONS(381), 1,
      sym_comment,
    ACTIONS(383), 1,
      anon_sym_COLON,
    ACTIONS(385), 1,
      sym_condition,
  [1837] = 3,
    ACTIONS(381), 1,
      sym_comment,
    ACTIONS(387), 1,
      anon_sym_COLON,
    ACTIONS(389), 1,
      sym_condition,
  [1847] = 3,
    ACTIONS(381), 1,
      sym_comment,
    ACTIONS(391), 1,
      anon_sym_COLON,
    ACTIONS(393), 1,
      sym_condition,
  [1857] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(130), 1,
      anon_sym_LBRACK,
    STATE(79), 1,
      sym_array,
  [1867] = 3,
    ACTIONS(381), 1,
      sym_comment,
    ACTIONS(395), 1,
      anon_sym_COLON,
    ACTIONS(397), 1,
      sym_condition,
  [1877] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(31), 1,
      anon_sym_LBRACK,
    STATE(79), 1,
      sym_array,
  [1887] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(399), 1,
      sym__terminator,
  [1894] = 2,
    ACTIONS(381), 1,
      sym_comment,
    ACTIONS(401), 1,
      sym_condition,
  [1901] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(403), 1,
      ts_builtin_sym_end,
  [1908] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(405), 1,
      sym__terminator,
  [1915] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(407), 1,
      anon_sym_COLON,
  [1922] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(409), 1,
      anon_sym_RBRACK,
  [1929] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(411), 1,
      sym__terminator,
  [1936] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(413), 1,
      sym__terminator,
  [1943] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(415), 1,
      sym__terminator,
  [1950] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(417), 1,
      sym__terminator,
  [1957] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(419), 1,
      sym_identifier,
  [1964] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(421), 1,
      sym__terminator,
  [1971] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(132), 1,
      sym_range,
  [1978] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(423), 1,
      anon_sym_COLON,
  [1985] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(425), 1,
      sym__terminator,
  [1992] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(427), 1,
      sym_identifier,
  [1999] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(429), 1,
      sym__terminator,
  [2006] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(431), 1,
      sym__terminator,
  [2013] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(433), 1,
      sym__terminator,
  [2020] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(435), 1,
      anon_sym_COLON,
  [2027] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(437), 1,
      sym__terminator,
  [2034] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(439), 1,
      sym__terminator,
  [2041] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(441), 1,
      sym_string,
  [2048] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(443), 1,
      sym__terminator,
  [2055] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(445), 1,
      sym__terminator,
  [2062] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(447), 1,
      sym__terminator,
  [2069] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(449), 1,
      sym__terminator,
  [2076] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(451), 1,
      anon_sym_COLON,
  [2083] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(453), 1,
      anon_sym_COLON,
  [2090] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(455), 1,
      anon_sym_COLON,
  [2097] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(457), 1,
      sym_identifier,
  [2104] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(459), 1,
      anon_sym_COLON,
  [2111] = 2,
    ACTIONS(381), 1,
      sym_comment,
    ACTIONS(461), 1,
      sym_condition,
  [2118] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(463), 1,
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
  [SMALL_STATE(10)] = 283,
  [SMALL_STATE(11)] = 319,
  [SMALL_STATE(12)] = 352,
  [SMALL_STATE(13)] = 387,
  [SMALL_STATE(14)] = 422,
  [SMALL_STATE(15)] = 452,
  [SMALL_STATE(16)] = 482,
  [SMALL_STATE(17)] = 512,
  [SMALL_STATE(18)] = 533,
  [SMALL_STATE(19)] = 554,
  [SMALL_STATE(20)] = 575,
  [SMALL_STATE(21)] = 596,
  [SMALL_STATE(22)] = 624,
  [SMALL_STATE(23)] = 652,
  [SMALL_STATE(24)] = 680,
  [SMALL_STATE(25)] = 708,
  [SMALL_STATE(26)] = 734,
  [SMALL_STATE(27)] = 757,
  [SMALL_STATE(28)] = 780,
  [SMALL_STATE(29)] = 803,
  [SMALL_STATE(30)] = 826,
  [SMALL_STATE(31)] = 849,
  [SMALL_STATE(32)] = 872,
  [SMALL_STATE(33)] = 895,
  [SMALL_STATE(34)] = 918,
  [SMALL_STATE(35)] = 941,
  [SMALL_STATE(36)] = 964,
  [SMALL_STATE(37)] = 987,
  [SMALL_STATE(38)] = 1010,
  [SMALL_STATE(39)] = 1033,
  [SMALL_STATE(40)] = 1056,
  [SMALL_STATE(41)] = 1079,
  [SMALL_STATE(42)] = 1102,
  [SMALL_STATE(43)] = 1125,
  [SMALL_STATE(44)] = 1141,
  [SMALL_STATE(45)] = 1157,
  [SMALL_STATE(46)] = 1173,
  [SMALL_STATE(47)] = 1189,
  [SMALL_STATE(48)] = 1205,
  [SMALL_STATE(49)] = 1221,
  [SMALL_STATE(50)] = 1237,
  [SMALL_STATE(51)] = 1253,
  [SMALL_STATE(52)] = 1269,
  [SMALL_STATE(53)] = 1285,
  [SMALL_STATE(54)] = 1300,
  [SMALL_STATE(55)] = 1315,
  [SMALL_STATE(56)] = 1334,
  [SMALL_STATE(57)] = 1353,
  [SMALL_STATE(58)] = 1368,
  [SMALL_STATE(59)] = 1383,
  [SMALL_STATE(60)] = 1398,
  [SMALL_STATE(61)] = 1417,
  [SMALL_STATE(62)] = 1432,
  [SMALL_STATE(63)] = 1447,
  [SMALL_STATE(64)] = 1462,
  [SMALL_STATE(65)] = 1477,
  [SMALL_STATE(66)] = 1492,
  [SMALL_STATE(67)] = 1507,
  [SMALL_STATE(68)] = 1522,
  [SMALL_STATE(69)] = 1537,
  [SMALL_STATE(70)] = 1552,
  [SMALL_STATE(71)] = 1567,
  [SMALL_STATE(72)] = 1582,
  [SMALL_STATE(73)] = 1601,
  [SMALL_STATE(74)] = 1620,
  [SMALL_STATE(75)] = 1640,
  [SMALL_STATE(76)] = 1660,
  [SMALL_STATE(77)] = 1680,
  [SMALL_STATE(78)] = 1691,
  [SMALL_STATE(79)] = 1702,
  [SMALL_STATE(80)] = 1713,
  [SMALL_STATE(81)] = 1723,
  [SMALL_STATE(82)] = 1733,
  [SMALL_STATE(83)] = 1743,
  [SMALL_STATE(84)] = 1753,
  [SMALL_STATE(85)] = 1763,
  [SMALL_STATE(86)] = 1773,
  [SMALL_STATE(87)] = 1784,
  [SMALL_STATE(88)] = 1795,
  [SMALL_STATE(89)] = 1806,
  [SMALL_STATE(90)] = 1817,
  [SMALL_STATE(91)] = 1827,
  [SMALL_STATE(92)] = 1837,
  [SMALL_STATE(93)] = 1847,
  [SMALL_STATE(94)] = 1857,
  [SMALL_STATE(95)] = 1867,
  [SMALL_STATE(96)] = 1877,
  [SMALL_STATE(97)] = 1887,
  [SMALL_STATE(98)] = 1894,
  [SMALL_STATE(99)] = 1901,
  [SMALL_STATE(100)] = 1908,
  [SMALL_STATE(101)] = 1915,
  [SMALL_STATE(102)] = 1922,
  [SMALL_STATE(103)] = 1929,
  [SMALL_STATE(104)] = 1936,
  [SMALL_STATE(105)] = 1943,
  [SMALL_STATE(106)] = 1950,
  [SMALL_STATE(107)] = 1957,
  [SMALL_STATE(108)] = 1964,
  [SMALL_STATE(109)] = 1971,
  [SMALL_STATE(110)] = 1978,
  [SMALL_STATE(111)] = 1985,
  [SMALL_STATE(112)] = 1992,
  [SMALL_STATE(113)] = 1999,
  [SMALL_STATE(114)] = 2006,
  [SMALL_STATE(115)] = 2013,
  [SMALL_STATE(116)] = 2020,
  [SMALL_STATE(117)] = 2027,
  [SMALL_STATE(118)] = 2034,
  [SMALL_STATE(119)] = 2041,
  [SMALL_STATE(120)] = 2048,
  [SMALL_STATE(121)] = 2055,
  [SMALL_STATE(122)] = 2062,
  [SMALL_STATE(123)] = 2069,
  [SMALL_STATE(124)] = 2076,
  [SMALL_STATE(125)] = 2083,
  [SMALL_STATE(126)] = 2090,
  [SMALL_STATE(127)] = 2097,
  [SMALL_STATE(128)] = 2104,
  [SMALL_STATE(129)] = 2111,
  [SMALL_STATE(130)] = 2118,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(9),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(112),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(119),
  [13] = {.entry = {.count = 1, .reusable = false}}, SHIFT(127),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(98),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(97),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [23] = {.entry = {.count = 1, .reusable = false}}, SHIFT(77),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(94),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(5),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(17),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(3),
  [47] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(18),
  [50] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(4),
  [53] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0),
  [55] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(5),
  [58] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(5),
  [61] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(17),
  [64] = {.entry = {.count = 1, .reusable = true}}, SHIFT(83),
  [66] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [68] = {.entry = {.count = 1, .reusable = false}}, SHIFT(8),
  [70] = {.entry = {.count = 1, .reusable = true}}, SHIFT(96),
  [72] = {.entry = {.count = 1, .reusable = true}}, SHIFT(81),
  [74] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [76] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [78] = {.entry = {.count = 1, .reusable = true}}, SHIFT(108),
  [80] = {.entry = {.count = 1, .reusable = false}}, SHIFT(108),
  [82] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [84] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [86] = {.entry = {.count = 1, .reusable = true}}, SHIFT(103),
  [88] = {.entry = {.count = 1, .reusable = false}}, SHIFT(103),
  [90] = {.entry = {.count = 1, .reusable = true}}, SHIFT(105),
  [92] = {.entry = {.count = 1, .reusable = false}}, SHIFT(105),
  [94] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [96] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [98] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [100] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [102] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(9),
  [105] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(112),
  [108] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(119),
  [111] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(127),
  [114] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(98),
  [117] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(13),
  [120] = {.entry = {.count = 1, .reusable = true}}, SHIFT(130),
  [122] = {.entry = {.count = 1, .reusable = false}}, SHIFT(130),
  [124] = {.entry = {.count = 1, .reusable = true}}, SHIFT(106),
  [126] = {.entry = {.count = 1, .reusable = false}}, SHIFT(106),
  [128] = {.entry = {.count = 1, .reusable = false}}, SHIFT(84),
  [130] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [132] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [134] = {.entry = {.count = 1, .reusable = false}}, SHIFT(82),
  [136] = {.entry = {.count = 1, .reusable = false}}, SHIFT(85),
  [138] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean, 1, 0, 0),
  [140] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean, 1, 0, 0),
  [142] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_null, 1, 0, 0),
  [144] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_null, 1, 0, 0),
  [146] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 2, 0, 0),
  [148] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 2, 0, 0),
  [150] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 3, 0, 0),
  [152] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 3, 0, 0),
  [154] = {.entry = {.count = 1, .reusable = false}}, SHIFT(10),
  [156] = {.entry = {.count = 1, .reusable = false}}, SHIFT(129),
  [158] = {.entry = {.count = 1, .reusable = false}}, SHIFT(92),
  [160] = {.entry = {.count = 1, .reusable = false}}, SHIFT(45),
  [162] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [164] = {.entry = {.count = 1, .reusable = false}}, SHIFT(91),
  [166] = {.entry = {.count = 1, .reusable = false}}, SHIFT(53),
  [168] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [170] = {.entry = {.count = 1, .reusable = false}}, SHIFT(95),
  [172] = {.entry = {.count = 1, .reusable = false}}, SHIFT(64),
  [174] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [176] = {.entry = {.count = 1, .reusable = false}}, SHIFT(93),
  [178] = {.entry = {.count = 1, .reusable = false}}, SHIFT(46),
  [180] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(10),
  [183] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(129),
  [186] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0),
  [188] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(25),
  [191] = {.entry = {.count = 1, .reusable = false}}, SHIFT(67),
  [193] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [195] = {.entry = {.count = 1, .reusable = false}}, SHIFT(50),
  [197] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [199] = {.entry = {.count = 1, .reusable = false}}, SHIFT(49),
  [201] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [203] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(9),
  [206] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(98),
  [209] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_object_block_repeat1, 2, 0, 0), SHIFT_REPEAT(30),
  [212] = {.entry = {.count = 1, .reusable = false}}, SHIFT(47),
  [214] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [216] = {.entry = {.count = 1, .reusable = false}}, SHIFT(51),
  [218] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [220] = {.entry = {.count = 1, .reusable = false}}, SHIFT(68),
  [222] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [224] = {.entry = {.count = 1, .reusable = false}}, SHIFT(52),
  [226] = {.entry = {.count = 1, .reusable = false}}, SHIFT(69),
  [228] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [230] = {.entry = {.count = 1, .reusable = false}}, SHIFT(48),
  [232] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [234] = {.entry = {.count = 1, .reusable = false}}, SHIFT(54),
  [236] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [238] = {.entry = {.count = 1, .reusable = false}}, SHIFT(65),
  [240] = {.entry = {.count = 1, .reusable = false}}, SHIFT(66),
  [242] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 3, 0, 3),
  [244] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 3, 0, 3),
  [246] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 4, 0, 6),
  [248] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 4, 0, 6),
  [250] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 5, 0, 8),
  [252] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 5, 0, 8),
  [254] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 6, 0, 8),
  [256] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 6, 0, 8),
  [258] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 8, 0, 8),
  [260] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 8, 0, 8),
  [262] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object_block, 4, 0, 5),
  [264] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object_block, 4, 0, 5),
  [266] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object_block, 5, 0, 5),
  [268] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object_block, 5, 0, 5),
  [270] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 9, 0, 8),
  [272] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 9, 0, 8),
  [274] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 10, 0, 8),
  [276] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 10, 0, 8),
  [278] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 11, 0, 8),
  [280] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_block, 11, 0, 8),
  [282] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [284] = {.entry = {.count = 1, .reusable = false}}, SHIFT(87),
  [286] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [288] = {.entry = {.count = 1, .reusable = false}}, SHIFT(62),
  [290] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [292] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata, 4, 0, 4),
  [294] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_metadata, 4, 0, 4),
  [296] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gather_statement, 5, 0, 7),
  [298] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_gather_statement, 5, 0, 7),
  [300] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_schema_block_repeat1, 2, 0, 0), SHIFT_REPEAT(2),
  [303] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_schema_block_repeat1, 2, 0, 0),
  [305] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_block_repeat1, 2, 0, 0), SHIFT_REPEAT(60),
  [308] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_block, 6, 0, 1),
  [310] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_block, 6, 0, 1),
  [312] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_block, 5, 0, 1),
  [314] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_block, 5, 0, 1),
  [316] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gather_statement, 3, 0, 2),
  [318] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_gather_statement, 3, 0, 2),
  [320] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata, 3, 0, 1),
  [322] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_metadata, 3, 0, 1),
  [324] = {.entry = {.count = 1, .reusable = false}}, SHIFT(88),
  [326] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [328] = {.entry = {.count = 1, .reusable = false}}, SHIFT(61),
  [330] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [332] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [334] = {.entry = {.count = 1, .reusable = true}}, SHIFT(109),
  [336] = {.entry = {.count = 1, .reusable = true}}, SHIFT(86),
  [338] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0), SHIFT_REPEAT(80),
  [341] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [344] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0), SHIFT_REPEAT(109),
  [347] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_schema_field_repeat1, 2, 0, 0),
  [349] = {.entry = {.count = 1, .reusable = true}}, SHIFT(89),
  [351] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_primitive, 1, 0, 0),
  [353] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_array_type, 3, 0, 0),
  [355] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_enum, 2, 0, 0),
  [357] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_modifier, 1, 0, 0),
  [359] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_modifier, 2, 0, 0),
  [361] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_field, 3, 0, 9),
  [363] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_field, 3, 0, 9),
  [365] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_object, 4, 0, 5),
  [367] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_object, 4, 0, 5),
  [369] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_object, 5, 0, 5),
  [371] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_object, 5, 0, 5),
  [373] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_schema_field, 4, 0, 9),
  [375] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_schema_field, 4, 0, 9),
  [377] = {.entry = {.count = 1, .reusable = true}}, SHIFT(107),
  [379] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [381] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [383] = {.entry = {.count = 1, .reusable = false}}, SHIFT(122),
  [385] = {.entry = {.count = 1, .reusable = true}}, SHIFT(128),
  [387] = {.entry = {.count = 1, .reusable = false}}, SHIFT(114),
  [389] = {.entry = {.count = 1, .reusable = true}}, SHIFT(125),
  [391] = {.entry = {.count = 1, .reusable = false}}, SHIFT(115),
  [393] = {.entry = {.count = 1, .reusable = true}}, SHIFT(116),
  [395] = {.entry = {.count = 1, .reusable = false}}, SHIFT(121),
  [397] = {.entry = {.count = 1, .reusable = true}}, SHIFT(126),
  [399] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [401] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [403] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [405] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [407] = {.entry = {.count = 1, .reusable = true}}, SHIFT(117),
  [409] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [411] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [413] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [415] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [417] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [419] = {.entry = {.count = 1, .reusable = true}}, SHIFT(104),
  [421] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [423] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [425] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [427] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [429] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [431] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [433] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [435] = {.entry = {.count = 1, .reusable = true}}, SHIFT(100),
  [437] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [439] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [441] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [443] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [445] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [447] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [449] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [451] = {.entry = {.count = 1, .reusable = true}}, SHIFT(120),
  [453] = {.entry = {.count = 1, .reusable = true}}, SHIFT(115),
  [455] = {.entry = {.count = 1, .reusable = true}}, SHIFT(122),
  [457] = {.entry = {.count = 1, .reusable = true}}, SHIFT(110),
  [459] = {.entry = {.count = 1, .reusable = true}}, SHIFT(123),
  [461] = {.entry = {.count = 1, .reusable = true}}, SHIFT(124),
  [463] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
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

TS_PUBLIC const TSLanguage *tree_sitter_rune(void) {
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
