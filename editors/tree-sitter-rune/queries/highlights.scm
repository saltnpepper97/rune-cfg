; Keywords
[
  "gather"
  "as"
  "if"
  "else"
  "elseif"
  "else-if"
  "schema"
  "with"
] @keyword

[
  "end"
] @keyword.return

[
  "required"
  "default"
  "range"
  "enum"
] @keyword.modifier

; Literals
(string) @string
(regex) @string.regexp
(number) @number
(range) @number
(boolean) @boolean
(null) @constant.builtin
(comment) @comment

; Names and keys
(metadata name: (identifier) @attribute)
(gather_statement alias: (identifier) @variable)
(assignment key: (identifier) @property)
(assignment key: (string) @property)
(inline_attribute name: (identifier) @property)
(inline_attribute name: (string) @property)
(object_block name: (identifier) @type)
(schema_block name: (identifier) @type.definition)
(schema_object name: (identifier) @type)
(object_block end_name: (identifier) @type)
(schema_block end_name: (identifier) @type.definition)
(schema_object end_name: (identifier) @type)
(schema_field name: (identifier) @property)
(schema_primitive) @type.builtin

; References
(reference) @variable.parameter

; Punctuation
[
  "@"
  ":"
  "="
  "["
  "]"
  ","
] @punctuation.delimiter
