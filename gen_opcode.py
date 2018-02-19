import sys, os

OP_TR = 0
OP_T1 = 1
OP_T2 = 2
OP_T3 = 3
OP_M = 4
OP_PREFIX = 5
OP_CODE = 6
OP_NAME = 7
OP_TEXT = 8

def gen_prologue(out):
    out.write("""use types::ValueType;

#[derive(Debug)]
pub struct Op {
    pub tr: Option<ValueType>,
    pub t1: Option<ValueType>,
    pub t2: Option<ValueType>,
    pub m: u8,
    pub code: u8,
    pub text: &'static str,
}

pub const ___: Option<ValueType> = None;
pub const I32: Option<ValueType> = Some(ValueType::I32);
pub const I64: Option<ValueType> = Some(ValueType::I64);
pub const F32: Option<ValueType> = Some(ValueType::F32);
pub const F64: Option<ValueType> = Some(ValueType::F64);

""")

def to_name(row):
    return row[OP_TEXT].upper().replace('.','_').replace('/','_').replace('"','')

def to_op_type(t):
    return t
    # if t == '___':
    #     return 'None'
    # else:
    #     return 'Some(ValueType::%s)' % t

def gen_opcodes(out, rows):
    for row in rows:
        out.write("pub const {:24}: u8 = {};\n".format(to_name(row), row[OP_CODE]))
    out.write("\n\n")

def gen_ops(out, rows):
    for row in rows:
        out.write("pub const {:24}: Op = Op {{".format(to_name(row)+'_OP'))
        out.write(" tr: %s, " % to_op_type(row[OP_TR]))
        out.write(" t1: %s, " % to_op_type(row[OP_T1]))
        out.write(" t2: %s, " % to_op_type(row[OP_T2]))
        # out.write(" t3: %s, " % to_op_type(row[OP_T3]))
        out.write(" m: %s, " % row[OP_M])
        # out.write(" prefix: %s, " % row[OP_PREFIX])
        out.write(" code: %s, " % row[OP_CODE])
        out.write(" text: %s, " % row[OP_TEXT])
        out.write("};\n")

def gen_op_from(out, rows):
    out.write("""
impl Op {
    pub fn from_opcode(opc: u8) -> Option<Op> {
        Some(match opc {
""")
    for row in rows:
        opc = to_name(row)
        op = to_name(row)+'_OP'
        out.write("            {:20} => {},\n".format(opc, op))
    out.write("            {:20} => {},\n".format('_', 'return None'))

    out.write("""        })
    }
}
""")

def gen_code(out, rows):
    gen_prologue(out)
    gen_opcodes(out, rows)
    gen_ops(out, rows)
    gen_op_from(out, rows)

def read_opcodes(f):
    rows = []
    for line in f:
        if line.find('WABT_OPCODE(') != 0:
            continue
        row = line.replace('WABT_OPCODE(','').replace(')','').split(',')
        row = [c.strip() for c in row]

        if row[OP_TR] == 'V128':
            continue
        if row[OP_PREFIX] != '0':
            continue
        rows.append(row)
    return rows

def main():
    with open('wabt_opcode.def') as f:
        with open('src/parser/opcode.rs','w') as out:
            gen_code(out, read_opcodes(f))

if __name__ == '__main__':
    main()