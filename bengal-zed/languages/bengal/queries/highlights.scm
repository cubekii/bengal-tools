(identifier) @variable
(type) @type

(keyword) @keyword
"fn" @keyword.function
"let" @keyword
"var" @keyword
"const" @keyword
"if" @keyword.control
"else" @keyword.control
"while" @keyword.control
"for" @keyword.control
"in" @keyword.control
"return" @keyword.control
"break" @keyword.control
"continue" @keyword.control
"import" @keyword.control
"as" @keyword.control
"class" @keyword.control
"private" @keyword.modifier
"try" @keyword.control
"catch" @keyword.control
"throw" @keyword.control
"self" @variable.builtin
"native" @variable.builtin

(boolean) @constant.builtin
"null" @constant.builtin

(number) @number
(string) @string
(multiline_string) @string
(escape_sequence) @constant.character.escape

(comment) @comment

(call_expression
  (identifier) @function)

(function_declaration
  name: (identifier) @function)

(class_declaration
  name: (identifier) @type)

(parameter
  name: (identifier) @variable.parameter)

(field_declaration
  name: (identifier) @property)

(member_access
  member: (identifier) @property)

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  ","
  "."
  ":"
  ";"
  "::"
] @punctuation.delimiter

[
  "="
  "=="
  "!="
  "<"
  "<="
  ">"
  ">="
  "+"
  "-"
  "*"
  "/"
  "%"
  "&&"
  "||"
  "!"
  ".."
  "->"
  "=>"
] @operator
