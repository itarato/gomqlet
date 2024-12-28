# Gomqlet

Toy graphql autocomplete project.

### Todo

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
- autocomplete to a vertical list
- autocomplete with fuzzy search
- only offer not yet added fields
- arglist value completion
- apply recommendation (trigger: shortcut?)
- validation:
    - arg requirement
    - fieldset
    - fieldset emptiness

GraphQL:
- schema config (config file or command line param)
- query/mutation over HTTP
- magic tokens (eg.: random string)
