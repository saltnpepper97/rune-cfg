; Keywords
[
  "gather"
  "as"
  "if"
  "else"
  "elseif"
  "else-if"
  "schema"
] @keyword

[
  "end"
  "endif"
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
(object_block name: (identifier) @type)
(schema_block name: (identifier) @type.definition)
(schema_object name: (identifier) @type)
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
