# Gomqlet

Toy graphql autocomplete project.

Features:

- GraphQL query / mutation editor
- syntax highlight
- tokenizer + parser
- autocompletion
- error detection
- query / mutation exection (over HTTP)

### Todo

Bug:
- if cursor is between arglist key and value the suggestion is affering the parent arglist keywords instead of being empty

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
- validation:
    - arg requirement
    - fieldset
    - fieldset emptiness

GraphQL:
- magic tokens (eg.: random string)
