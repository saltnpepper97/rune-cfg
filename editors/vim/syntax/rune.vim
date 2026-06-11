" Vim syntax file
" Language: RUNE

if exists("b:current_syntax")
  finish
endif

syn case match

syn match runeComment "#.*$"

syn region runeString start=+"+ skip=+\\"+ end=+"+ contains=runeDollarReference
syn region runeString start=+'+ skip=+\\'+ end=+'+ contains=runeDollarReference
syn region runeRegex start=+r"+ skip=+\\"+ end=+"+

syn match runeNumber "\v<\d+(\.\d+)?>"
syn keyword runeBoolean true false
syn keyword runeNull null None

syn keyword runeKeyword gather as if else elseif else-if schema
syn keyword runeEndKeyword end endif
syn keyword runeSchemaKeyword required default range
syn keyword runeSchemaType string str int integer float number bool boolean regex any object

syn match runeMetadata "@[A-Za-z_][A-Za-z0-9_-]*"
syn match runeDollarReference "\$\(env\|sys\|runtime\|var\)\(\.[A-Za-z_][A-Za-z0-9_-]*\)*"

syn match runeTopLevelObjectKey "^[A-Za-z_][A-Za-z0-9_-]*\ze\s*:"
syn match runeObjectKey "^\s\+[A-Za-z_][A-Za-z0-9_-]*\ze\s*:"
syn match runeKey "^\s*[A-Za-z_][A-Za-z0-9_-]*\ze\s*[= ]"
syn match runeObjectDelimiter "[:\[\]]"

hi def link runeComment Comment
hi def link runeString Character
hi def link runeRegex Special
hi def link runeNumber Number
hi def link runeBoolean Boolean
hi def link runeNull Constant
hi def link runeKeyword Statement
hi def link runeEndKeyword Delimiter
hi def link runeSchemaKeyword Keyword
hi def link runeSchemaType Type
hi def link runeMetadata PreProc
hi def link runeDollarReference Special
hi def link runeTopLevelObjectKey Structure
hi def link runeObjectKey Function
hi def link runeKey Identifier
hi def link runeObjectDelimiter Delimiter

let b:current_syntax = "rune"
