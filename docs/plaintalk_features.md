# PlainTalk Features

PlainTalk is a programming language designed to be simple and intuitive, using natural language constructs.

The main goal of PlainTalk is to make programming as accessible as possible. Therefore, the main precepts of PlainTalk are:

1. It must be easily readable to almost anyone, even those with no programming experience. The purpose of the syntax is to expose the logic of the code, and not hide it behind "complex-looking"—albeit concise—syntax.
2. It needs to eliminate implicit, behind-the-scenes behavior entirely. Every action that the compiler does must be explicitly defined by the user.
3. It should enforce good programming practices.
4. It should be as concise as possible without sacrificing 1 or 2. This means eliminating unnecessary keywords.
5. Every statement line must be a complete sentence. Because why not.

Notice that nowhere in these precepts is there a mention of "easily writable", "fully-featured", and so on. If you want that, this is NOT the language for you. PlainTalk is not meant to be an "industry" language that is conducive to proper, professional work, and rather, it is meant to be a language that is easy to read and understand, even for those who have never programmed before. Therefore, many of the features that are common in other languages are not present in PlainTalk.

## Object Files

An object file is akin to classes in other languages, minus inheritance and polymorphism. No special syntax is required to define an object file, one must simply create a file with the `.pt` extension. The file name is used as the object name. All files in PlainTalk are object files.

A main.pt object file is a necessary object file whose main function is called as the entry point of the program.

### Object File Contents

Object files can contain [variables](#variables) and [functions](#functions), and can be instantiated to create objects.

### Instancing an Object File

Object files can be instantiated the same way as classes in other languages, using the following syntax:

```
new <file_name_without_extension>
```

This will return an instance of the file with the default values of its variables. Constructors can be defined explicitly within the file and called after instantiation:

```
creare integer variable my_class_instance = new my_class.
call my_class_instance's constructor with 10.
```

### Accessing variables and functions

To access variables and functions within a file, you can use the following syntax:

```
<file_name>'s <member_name>
```

## Functions

### Creating a Function

```
create [<keyword1> <keyword2> ...] function <name>:
	[create input <type> variable <name> with <default_value>.]
	[create input <type> variable <name> with <default_value>.]
	[...]
	[create output <type> variable <name> with <default_value>.]

	<function body>

	[return.]
```

It is important to note a few things about function definitions:
1. The function body must be indented, similar to python.
2. Inputs (or "arguments" in other languages) are defined at the beginning of the function definition, and must be given an initial value (like regular variables.)
3. The value that the function returns is stored in an output variable, which is also defined at the beginning of the function definition. Only one output variable is allowed per function. When the function is finished, whatever the output variable contains will be returned to the caller.
4. The `return` keyword is optional and doesn't explicitly return anything, it is simply used to break out of the function.

### Calling a Function

```
call <function_name> [with <arg1>, <arg2>, ... and <argN>].
```

> Note: Function calls are only allowed to be either a value or a variable. This is not because I am a bad or lazy programmer. Well, it's unrelated to that at least. I had to make trade-offs between maintaining the readable english-like syntax, making it as concise as possible, and making it functional. Allowing complex expressions as function arguments can introduce highly unreadable syntax. The remedy is to store all function calls and expressions in variables before passing them to other functions. This is a good practice in any language, and it has the added benefit of making the code more readable.

#### File Keywords

- **public**: The function is accessible from outside the file (default).
- **private**: The function is only accessible within the file.

## Variables

Variables in PlainTalk are used to store data. They can be defined at the file level or within functions. Variables can be of various types, such as integer, string, boolean, etc.

### Creating a Variable

```
create <keywords> <variable_type> variable <variable_name> with <initial_value>.
```


#### File Keywords
- **public**: The variable is accessible from outside the file (default).
- **private**: The variable is only accessible within the file.

#### Function Keywords
- **input**: Defines an input variable for the function. Must be at the beginning of the function definition.
- **output**: Defines an output variable for the function. Must be at the beginning of the function definition.

#### Loop Keywords
- **index**: Defines an index variable for a loop. Must be at the beginning of the loop definition.

### Setting a Variable

To set a variable to a new value, you can use the following syntax:

```
change <variable_name> to <value>.
```

### Incrementing a Variable

- To increment a variable, you can use the following syntax:

```
change <variable_name> by <value>.
```

## Comments

Comments in PlainTalk are used to explain the code and make it more readable. They can be single-line or multi-line. They will always, whether single-line or multi-line, be inside a pair of square brackets (`[]`). Since their ending point is defined, they can be placed anywhere in the code, even in the middle of a line:

```
create integer [this function is an integer because we don't need more resolution] variable degrees with 10.
```

## Control Statements

PlainTalk supports a limited set of control statements to manage the flow of execution in your code.

### If Statements

```
if <condition> is <true/false>:
	<code block>
otherwise if <condition> is <true/false>:
	<code block>
otherwise:
	<code block>
```

In order to increase readability, PlainTalk forces the user to choose whether to evaluate a condition against `true` or `false`, rather than just evaluating the condition against `true` as in most languages. The thought behind this is that it makes it more easily readable at a glance. For example,

```
if value is false:
	<code block>
```

is more readable than

```
if not value:
	<code block>
```

Therefore, the `if` statement must always be followed by `is true` or `is false`, and the same applies to the `otherwise if` statement. The `otherwise` statement does not require a condition, as it is the default case.

### Repeat Statements

```
repeat <number> times:
	[create index integer variable <index_name> with 0.]
	<code block>
```

Notably, an optional index variable, much like the input/output variables in functions, can be defined at the beginning of the repeat statement. This variable will be incremented automatically with each iteration of the loop, starting from 0. It must be declared at the beginning of the repeat statement, with something other than `nothing` as its initial value. It must also be defined as an integer variable.

### While Statements

```
while <condition> == <true/false>:
	<code block>
```

Same as the `if` statement, the `while` statement allows you to choose whether to evaluate the condition against `true` or `false`.

## Line Delimiters

As you may have noticed, PlainTalk uses a period (`.`) to indicate the end of a line, similar to how other languages use semicolons (`;`). This is done to make the code look more like natural language. It also allows for [multi-line statements](#multi-line-statements).

## Keywords

- **create**: Used to define variables, functions, and other constructs.
- **call**: Used to invoke functions.
- **with, and**: Used to pass arguments to functions. The last argument is preceded by `and` to indicate the end of the argument list.
- **with** In the context of variables, assigns a value to a variable on its initialization.
- **look-up**: Used to access elements in collections (lists and dictionaries).
- **new**: Used to create an instance of a file (class).
- **return**: Used to exit a function.
- **as** convert a value to a different type.
- **change ... by/to**: Used to modify the value of a variable.

### Math Keywords:
- **==**: Used to compare two values for equality.
- **is**: Used to compare two values for equality, similar to `==`, but with a more natural language feel.
- **!=**: Used to compare two values for inequality.
- **<**: Less than operator.
- **<=**: Less than or equal to operator.
- **>**: Greater than operator.
- **>=**: Greater than or equal to operator.
- **%**: Modulus operator.
- **^**: Exponentiation operator.
- **+**: Addition operator.
- **-**: Subtraction operator.
- **\***: Multiplication operator.
- **/**: Division operator.
- **and**: boolean and operator.
- **or**: boolean or operator.
- **not**: boolean not operator.

call function with, 10, 20 and 30, and 40.


### Multi-line Statements

Multi-line statements are supported in PlainTalk, as long as they are properly indented. For example:

```
call my_function with
	10,
	20,
	and 30.
```

Multi-line statements must have their subsequent lines indented one level deeper than the first line, and the last line must end with a period (`.`) to indicate the end of the statement.

## Types

PlainTalk supports a small set of basic types.

### Primitives

- **integer**: A whole number, e.g. `10`, `-5`. 64-bit signed integers are used.
- **float**: A floating-point number, e.g. `3.14`, `-0.001`. 64-bit double-precision floating-point numbers are used.
- **string**: A sequence of characters, e.g. `"Hello, World!"`, `"PlainTalk"`. Strings are enclosed in double quotes.
- **boolean**: A true or false value, e.g. `true`, `false`.
- **general**: A special type that can hold any value of any type.
- **nothing**: A special value that represents the absence of a value. It is used to indicate that a variable has not been initialized or that a function does not return a value. Any variable of any type can be set to `nothing`.

### Collections

- `<type>-list`: An ordered collection of values, e.g. `{1, 2, 3}`, `{"apple", "banana", "cherry"}`. Lists can contain values of any type, including other lists.
- `<key_type>-<value_type>-dictionary`: A collection of key-value pairs.

#### Indexing collections

To access elements in a collection, you can use the following syntax:

```
[Dictionaries]
look-up <key> in <dictionary_name>
[List]
look-up <index> in <list_name>
```

> Note: Like function calls, PlainTalk only supports simple values or variables as keys or indices. See the note in the [Function Calls](#calling-a-function) section for more details.

### Files

Files can be used as types, similar to classes in other languages.

## Naming Conventions

All names should be written in snake_case, including file names, variable names, and function names. For example, `my_variable`, `my_function`, and `my_file.pt`.