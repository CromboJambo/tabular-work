#!/usr/bin/env python3
"""Extract expression language from table.rs into expr/ module files."""

TABLE_PATH = "/home/crombo/projects/experiments/tabular-work/rsf-core/src/table.rs"
EXPR_DIR = "/home/crombo/projects/experiments/tabular-work/rsf-core/src/expr"

def read_file(path):
    with open(path, 'r') as f:
        return f.read()

def write_file(path, content):
    with open(path, 'w') as f:
        f.write(content)

table_src = read_file(TABLE_PATH)
lines = table_src.split('\n')

print(f"Original table.rs: {len(lines)} lines")

# Expression language spans lines 14-890 (indices 13-889)
# Types: Token(14), Expr(71), BinOp(95), BinLogOp(119), ParseError(134)
# Lexer: Tokenizer struct/impl (145-374)
# Parser: Parser struct/impl (375-684)  
# Evaluator: Expr methods (686-890)

types_lines = lines[13:144]  # Token through ParseError
lexer_lines = lines[144:374]  # Tokenizer struct and impl
parser_lines = lines[374:685]  # Parser struct and impl
evaluator_lines = lines[685:890]  # Expr methods

print(f"types lines: {len(types_lines)}")
print(f"lexer lines: {len(lexer_lines)}")
print(f"parser lines: {len(parser_lines)}")
print(f"evaluator lines: {len(evaluator_lines)}")

# Write types.rs
write_file(f"{EXPR_DIR}/types.rs", "\n".join(types_lines))

# Write lexer.rs
write_file(f"{EXPR_DIR}/lexer.rs", "\n".join(lexer_lines))

# Write parser.rs  
write_file(f"{EXPR_DIR}/parser.rs", "\n".join(parser_lines))

# Write evaluator.rs
write_file(f"{EXPR_DIR}/evaluator.rs", "\n".join(evaluator_lines))

print("Expression module files written to expr/")