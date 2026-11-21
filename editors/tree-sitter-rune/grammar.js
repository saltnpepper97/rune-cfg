const IDENTIFIER = /[A-Za-z_][A-Za-z0-9_-]*/;

module.exports = grammar({
  name: 'runecfg',

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

    // An identifier right after `end` is the block's optional label.
    schema_block: $ => prec.right(seq(
      'schema',
      field('name', $.identifier),
      ':',
      $._terminator,
      repeat(choice($._schema_item, $._terminator)),
      'end',
      optional(field('end_name', $.identifier)),
    )),

    _schema_item: $ => choice(
      $.schema_object,
      $.schema_field,
    ),

    // An identifier right after `end` is the block's optional label.
    schema_object: $ => prec.right(seq(
      field('name', $.identifier),
      ':',
      $._terminator,
      repeat(choice($._schema_item, $._terminator)),
      'end',
      optional(field('end_name', $.identifier)),
    )),

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

    // An identifier right after `end` is the block's optional label.
    object_block: $ => prec.right(seq(
      field('name', $.identifier),
      ':',
      $._terminator,
      repeat(choice($._object_item, $._terminator)),
      'end',
      optional(field('end_name', $.identifier)),
    )),

    _object_item: $ => choice(
      $.assignment,
      $.object_block,
      $.if_block,
    ),

    // `if` / any number of `elseif` / an optional `else`, closed by one `end`.
    if_block: $ => seq(
      'if',
      field('condition', $.condition),
      ':',
      $._terminator,
      repeat(choice($._object_item, $._terminator)),
      repeat(seq(
        choice('elseif', 'else-if'),
        field('condition', $.condition),
        ':',
        $._terminator,
        repeat(choice($._object_item, $._terminator)),
      )),
      optional(seq(
        'else',
        ':',
        $._terminator,
        repeat(choice($._object_item, $._terminator)),
      )),
      'end',
    ),

    assignment: $ => seq(
      field('key', choice($.identifier, $.string)),
      optional('='),
      field('value', $._value),
      optional(field('attributes', $.inline_attributes)),
      $._terminator,
    ),

    inline_attributes: $ => seq(
      'with',
      repeat1($.inline_attribute),
    ),

    inline_attribute: $ => seq(
      field('name', choice($.identifier, $.string)),
      field('value', $._value),
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
