# Gomqlet

Toy graphql autocomplete project.

Features:

- GraphQL query / mutation editor
- syntax highlight
- tokenizer + parser
- autocompletion
- (basic) error detection
- query / mutation exection (over HTTP)

## Use

```bash
cargo run -- -h
```

Bindings:

- `CTRL-O` and then `DIGIT` / `ALT-<DIGIT>`: apply suggestion
- `CTRL-G`: execute query
- `CTRL-C`: quit application

## Todo

Bug:

- some suggestion shows after closing braces

Editor:

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
- load schema realtime

GraphQL:

- magic tokens (eg.: random string)
