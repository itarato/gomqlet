# Gomqlet

Toy graphql executor project.

![Screenshot](./misc/screenshot.png)

![Screenshot](./misc/screenshot2.png)

Features:

- GraphQL query / mutation editor
- syntax highlight
- tokenizer + parser
- autocompletion
- (basic) error detection
- query / mutation exection (over HTTP)
- file browser
- query chaining (with result insertion)


## Use

```bash
cargo run -- -h
```


### Bindings

- `CTRL-O` and then `DIGIT` / `ALT-<DIGIT>` / `OPTION-<DIGIT>`: apply suggestion
- `CTRL-G`: execute query
- `CTRL-C` / `CTRL-D`: quit application
- `ALT-F` / `CTRL-F`: file browser
- `ALT-S` / `CTRL-S`: save file


### Magic tokens

Format: `<` + definition + `>`

Types:

- Query chaining: `query::FILE_NAME::JSON_PATH` (example: `<query::users.graphql::$.data.users.edges[0].node.name>`)
- Random string: `random_string::LENGTH` (example: `<random_string::10>`)
- Random integer: `random_integer::MIN::MAX` (example: `<random_integer::0::100>`)
- Random word: `random_word` (example: `<random_word>`)
- Variable (defined in *.config.json): `variable::JSON_PATH` (example: `<variable::$.user_query.tags[0]>`)

### File browser

Only `.graphql` file types (text/plain) can be opened.


## Todo

- only offer not yet added fields
- query validation:
    - arg requirement
    - fieldset
    - fieldset emptiness
- show parse error position
- input nested with non nul is not unwrapped - complains that input object type is expected
- `_` for keyword start
- numbers for keywords (not start)
- increase query time limit
- offer variable preview
