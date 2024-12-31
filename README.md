# Gomqlet

Toy graphql autocomplete project.

Features:

- GraphQL query / mutation editor
- syntax highlight
- tokenizer + parser
- autocompletion
- error detection
- query / mutation exection (over HTTP)

## Todo

Bug:

- some suggestion shows after closing braces

Editor:

- enter on an opening paren/brace/bracket should be +1 indent
- adding an opening parent/brace/bracket should add the closing one too
- save to file

Tokenizier:

-

Parsing:

-

Analysis:

- autocomplete with fuzzy search
- only offer not yet added fields
- arglist value completion
- show numbers on suggestions after CTRL-O
- validation:
    - arg requirement
    - fieldset
    - fieldset emptiness
- add type indicator to suggestion list (obj, list, enum, etc)

GraphQL:

- magic tokens (eg.: random string)
