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

syn keyword runeKeyword gather as if else elseif else-if schema with
syn keyword runeEndKeyword end
syn keyword runeSchemaKeyword required default range
syn keyword runeSchemaType string str int integer float number bool boolean regex any object

syn match runeMetadata "@[A-Za-z_][A-Za-z0-9_-]*"
syn match runeDollarReference "\$\(env\|sys\|runtime\|var\)\(\.[A-Za-z_][A-Za-z0-9_-]*\)*"

syn match runeTopLevelObjectKey "^[A-Za-z_][A-Za-z0-9_-]*\ze\s*:"
" Indented keywords are not keys, though keys may start with their letters.
syn match runeObjectKey "^\s\+\%(\%(if\|elseif\|else-if\|else\|end\)[A-Za-z0-9_-]\@!\)\@![A-Za-z_][A-Za-z0-9_-]*\ze\s*:"
syn match runeKey "^\s*\%(\%(if\|elseif\|else-if\|else\|end\)[A-Za-z0-9_-]\@!\)\@![A-Za-z_][A-Za-z0-9_-]*\ze\s*[= ]"
" `not` and `in` are keywords only as condition operators.
syn match runeConditionKeyword "\v(<(if|elseif|else-if|not)\s+)@<=not(\s+[A-Za-z_$])@="
syn match runeConditionKeyword "\v([A-Za-z0-9_-]\s+)@<=in(\s*\[)@="
syn match runeObjectDelimiter "[:\[\]]"
" The optional block name in `end <name>`.
syn match runeEndLabel "\v(^\s*end\s+)@<=([A-Za-z_][A-Za-z0-9_-]*|\"([^\"\\]|\\.)*\")"

hi def link runeComment Comment
hi def link runeString Character
hi def link runeRegex Special
hi def link runeNumber Number
hi def link runeBoolean Boolean
hi def link runeNull Constant
hi def link runeKeyword Statement
hi def link runeConditionKeyword Statement
hi def link runeEndKeyword Delimiter
hi def link runeSchemaKeyword Keyword
hi def link runeSchemaType Type
hi def link runeMetadata PreProc
hi def link runeDollarReference Special
hi def link runeTopLevelObjectKey Structure
hi def link runeObjectKey Function
hi def link runeEndLabel Function
hi def link runeKey Identifier
hi def link runeObjectDelimiter Delimiter

let b:current_syntax = "runecfg"
