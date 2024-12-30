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
- 

Editor:
- enter on an opening paren/brace/bracket should be +1 indent
- adding an opening parent/brace/bracket should add the closing one too
- load from file
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
- schema config (config file or command line param)
- magic tokens (eg.: random string)
