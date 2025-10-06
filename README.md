# vpl-parser-generator
Rust parser and translator generator for the language class of Visibly Pushdown Grammars.
This library is part of an academic research paper for a Bachelor Thesis, which will be linked after it is released.
This is currently not published on https://crates.io.

## Features
The library can generate a parser and translator for a subset of Visibly Pushdown Languages. These both work with linear time complexity. 

## Syntax
A grammar can be specified in the following way (translation omitted when it is not necessary for the explanation):
```
A:                                         // Nonterminal
  "[abc]*" -> ...                          // All features of rust regular expressions can be used
  ["\(" A=A "\)"] -> ...                   // Nested call and return, allowing for recursion
  B -> ...                                 // An identifier can be the only item of a rule
  "abc" "abc" B=B -> ...                   // A rule can be one or more items, where an item is either a regular expression, a nested call and return, or an identifier
  "abc" B=B -> B "abc"                     // Translation rules are defined after the '->'. Nonterminals need to be followed by an identifier to be used in the translation rule (NT=ID ... -> ID)
  
B:                                         // New nonterminal
  "\[(?P<value>.*)\]" -> "(" value ")"     // Capture groups can be used to use part of a captured expression in the translation
```
These rules have the following restrictions:
* A nonterminal may only be used as final item of a rule. Therefore, after a nonterminal, there can be no other nonterminals/regular expressions. The exception is within a nested call/return block.
* Between a nested call/return, only a nonterminal is allowed, not multiple items.
* All identifiers in the lhs of the grammar rule need to be used in the rhs.

## Restrictions
There are currently some restrictions which people intending to use the application should know:
* The application has some restrictions on identifier placement. Due to these restrictions **not all visibly pushdown languages can be modeled**. 
* At the start of an identifier, the recognizer and translator will chose the first matching regular expression, even if it will fail later. Thus, for the following grammar for example:
```
A:
  "a" "b" -> "b"
  "a" "c" -> "c"
```
will fail on the input "ac", as the first regular expression of the first rule matches the first character.
