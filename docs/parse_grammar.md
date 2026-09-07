# Parse Grammar

I wrote my own SLR1 parser for lox. There is an accompanying `.lparse` grammar for describing your grammar.

Major credit to LALR1Pop - I have copied much of the `.lalrpop` grammar.

```
# token, not production
embedded_rust := '{''{' [\u{0}-\u{255}]* '}''}'

# the first ident for both of these is the bound name
bound_leaf := '[' 'Ident' ':' 'Ident' ']';
bound_rule :=  '<' 'Ident' ':' 'Ident' '>';

node := bound_leaf | bound_rule;

production_definition := node* '=' '>' embedded_rust ','

rule := 'Ident' ':' 'Ident' '=' '{' production_definition* '}' ';'

set_goal_rule := 'goal_rule' '=' 'Ident' ';'
set_token_type := 'token_type' '=' 'Ident' ';'

grammar := set_goal_rule set_token_type rule*
```
