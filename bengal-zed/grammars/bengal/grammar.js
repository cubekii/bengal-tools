module.exports = grammar({
  name: 'bengal',

  extras: $ => [
    /\s/,
    $.comment,
  ],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._statement),

    _statement: $ => choice(
      $.import_statement,
      $.use_statement,
      $.class_declaration,
      $.function_declaration,
      $.variable_declaration,
      $.assignment,
      $.expression_statement,
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.match_statement,
      $.return_statement,
      $.break_statement,
      $.continue_statement,
      $.try_statement,
      $.throw_statement,
      $.block
    ),

    use_statement: $ => seq(
      'use',
      $.module_path
    ),

    match_statement: $ => seq(
      'match',
      '(', $.expression, ')',
      '{',
      repeat($.match_arm),
      '}'
    ),

    match_arm: $ => seq(
      $.expression,
      '=>',
      $._statement
    ),

    import_statement: $ => seq(
      'import',
      $.module_path,
      optional(seq('as', $.identifier))
    ),

    module_path: $ => sep1($.identifier, '::'),

    class_declaration: $ => seq(
      'class',
      field('name', $.identifier),
      '{',
      repeat(choice($.field_declaration, $.function_declaration)),
      '}'
    ),

    field_declaration: $ => seq(
      optional('private'),
      field('name', $.identifier),
      ':',
      $.type,
      optional(seq('=', $.expression))
    ),

    function_declaration: $ => seq(
      optional('private'),
      'fn',
      field('name', $.identifier),
      $.parameter_list,
      optional(seq(':', $.type)),
      $.block
    ),

    parameter_list: $ => seq(
      '(',
      sep($.parameter, ','),
      ')'
    ),

    parameter: $ => seq(
      field('name', $.identifier),
      ':',
      $.type
    ),

    variable_declaration: $ => seq(
      choice('let', 'var', 'const'),
      $.identifier,
      optional(seq(':', $.type)),
      optional(seq('=', $.expression))
    ),

    assignment: $ => seq(
      $._assignment_target,
      choice('=', '+=', '-=', '*=', '/=', '%='),
      $.expression
    ),

    _assignment_target: $ => choice(
      $.identifier,
      $.member_access
    ),

    expression_statement: $ => $.expression,

    if_statement: $ => seq(
      'if',
      '(', $.expression, ')',
      $._statement,
      optional(seq('else', $._statement))
    ),

    while_statement: $ => seq(
      'while',
      '(', $.expression, ')',
      $._statement
    ),

    for_statement: $ => seq(
      'for',
      '(', $.identifier, 'in', $.expression, ')',
      $._statement
    ),

    return_statement: $ => seq(
      'return',
      optional($.expression)
    ),

    break_statement: $ => 'break',
    continue_statement: $ => 'continue',

    try_statement: $ => seq(
      'try',
      $.block,
      repeat($.catch_clause)
    ),

    catch_clause: $ => seq(
      'catch',
      '(', $.identifier, ')',
      $.block
    ),

    throw_statement: $ => seq(
      'throw',
      $.expression
    ),

    block: $ => seq(
      '{',
      repeat($._statement),
      '}'
    ),

    expression: $ => choice(
      $.binary_expression,
      $.unary_expression,
      $.call_expression,
      $.member_access,
      $.range_expression,
      $._primary_expression
    ),

    range_expression: $ => prec.left(1, seq(
      $.expression,
      '..',
      $.expression
    )),

    binary_expression: $ => choice(
      ...[
        ['||', 1],
        ['&&', 2],
        ['==', 3], ['!=', 3],
        ['<', 4], ['<=', 4], ['>', 4], ['>=', 4],
        ['+', 5], ['-', 5],
        ['*', 6], ['/', 6], ['%', 6],
      ].map(([operator, precedence]) =>
        prec.left(precedence, seq(
          $.expression,
          operator,
          $.expression
        ))
      )
    ),

    unary_expression: $ => choice(
      prec.left(7, seq('!', $.expression)),
      prec.left(7, seq('-', $.expression)),
      prec.left(7, seq($.expression, '++')),
      prec.left(7, seq($.expression, '--'))
    ),

    call_expression: $ => prec(8, seq(
      field('function', $.expression),
      '(',
      sep($.expression, ','),
      ')'
    )),

    member_access: $ => prec(9, seq(
      field('object', $.expression),
      '.',
      field('member', $.identifier)
    )),

    _primary_expression: $ => choice(
      $.identifier,
      $.number,
      $.string,
      $.multiline_string,
      $.boolean,
      'null',
      'self',
      seq('(', $.expression, ')')
    ),

    type: $ => seq(
      choice(
        $.identifier,
        'int', 'float', 'bool', 'char', 'str', 'void', 'any'
      ),
      optional('?')
    ),

    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    number: $ => /\d+(\.\d+)?/,

    boolean: $ => choice('true', 'false'),

    string: $ => seq(
      '"',
      repeat(choice(
        $.string_content,
        $.interpolation,
        $.escape_sequence
      )),
      '"'
    ),

    string_content: $ => /[^"\\$]+/,
    escape_sequence: $ => /\\./,

    multiline_string: $ => seq(
      '"""',
      repeat(choice(
        $.multiline_string_content,
        $.interpolation,
        $.escape_sequence
      )),
      '"""'
    ),

    multiline_string_content: $ => /[^"\\$]+|"(?!"")/,

    interpolation: $ => seq(
      '${',
      $.expression,
      '}'
    ),

    comment: $ => choice(
      seq('//', /.*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')
    ),
  }
});

function sep(rule, separator) {
  return optional(sep1(rule, separator));
}

function sep1(rule, separator) {
  return seq(rule, repeat(seq(separator, rule)));
}
