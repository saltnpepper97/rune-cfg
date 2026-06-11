const IDENTIFIER = /[A-Za-z_][A-Za-z0-9_-]*/;

module.exports = grammar({
  name: 'rune',

  extras: $ => [/[ \t\r]/, $.comment],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat(choice($._item, $._terminator)),

    _item: $ => choice(
      $.metadata,
      $.schema_block,
      $.gather_statement,
      $.object_block,
      $.if_block,
      $.assignment,
    ),

    comment: _ => token(seq('#', /.*/)),

    metadata: $ => seq(
      '@',
      field('name', $.identifier),
      optional(field('value', $._value)),
      $._terminator,
    ),

    gather_statement: $ => seq(
      'gather',
      field('path', $.string),
      optional(seq('as', field('alias', $.identifier))),
      $._terminator,
    ),

    schema_block: $ => seq(
      'schema',
      field('name', $.identifier),
      ':',
      $._terminator,
      repeat(choice($._schema_item, $._terminator)),
      'end',
    ),

    _schema_item: $ => choice(
      $.schema_object,
      $.schema_field,
    ),

    schema_object: $ => seq(
      field('name', $.identifier),
      ':',
      $._terminator,
      repeat(choice($._schema_item, $._terminator)),
      'end',
    ),

    schema_field: $ => seq(
      field('name', $.identifier),
      field('type', $._schema_type),
      repeat($.schema_modifier),
      $._terminator,
    ),

    _schema_type: $ => choice(
      $.schema_primitive,
      $.schema_enum,
      $.schema_array_type,
    ),

    schema_primitive: _ => choice(
      'string',
      'str',
      'int',
      'integer',
      'float',
      'number',
      'bool',
      'boolean',
      'regex',
      'null',
      'any',
      'object',
    ),

    schema_enum: $ => seq('enum', $.array),

    schema_array_type: $ => seq('[', $._schema_type, ']'),

    schema_modifier: $ => choice(
      'required',
      seq('default', $._value),
      seq('range', $.range),
    ),

    object_block: $ => seq(
      field('name', $.identifier),
      ':',
      $._terminator,
      repeat(choice($._object_item, $._terminator)),
      'end',
    ),

    _object_item: $ => choice(
      $.assignment,
      $.object_block,
      $.if_block,
    ),

    if_block: $ => seq(
      'if',
      field('condition', $.condition),
      ':',
      $._terminator,
      repeat(choice($._object_item, $._terminator)),
      optional(seq(
        choice('else', 'elseif', 'else-if'),
        optional($.condition),
        ':',
        $._terminator,
        repeat(choice($._object_item, $._terminator)),
      )),
      'endif',
    ),

    assignment: $ => seq(
      field('key', $.identifier),
      optional('='),
      field('value', $._value),
      $._terminator,
    ),

    _value: $ => choice(
      $.conditional_value,
      $.string,
      $.regex,
      $.number,
      $.boolean,
      $.null,
      $.array,
      $.reference,
    ),

    conditional_value: _ => token(seq('if', /[^\n#]+/, 'else', /[^\n#]+/)),

    array: $ => seq(
      '[',
      repeat(choice($._value, ',', $._terminator)),
      ']'
    ),

    condition: _ => token(/[^:\n#]+/),

    reference: _ => token(seq(
      optional('$'),
      IDENTIFIER,
      repeat(seq('.', IDENTIFIER)),
    )),

    identifier: _ => token(IDENTIFIER),

    string: _ => token(choice(
      seq('"', repeat(choice(/[^"\\]/, /\\./)), '"'),
      seq("'", repeat(choice(/[^'\\]/, /\\./)), "'"),
    )),

    regex: _ => token(seq('r"', repeat(choice(/[^"\\]/, /\\./)), '"')),

    number: _ => token(/[+-]?\d+(\.\d+)?/),

    range: _ => token(/[+-]?\d+(\.\d+)?\.\.[+-]?\d+(\.\d+)?/),

    boolean: _ => choice('true', 'false'),

    null: _ => choice('null', 'None'),

    _terminator: _ => /\n+/,
  }
});
