import instructions from "../all.json"

const ins = Object.values(instructions)
    .filter(ins => ins.name)

const hexes = Array.from(new Set(ins.map(ins => ins.hex)));
const clks = Array.from(new Set(ins.map(ins => ins.clk)));
const names = Array.from(new Set(ins.map(ins => ins.name)));
const bytes = Array.from(new Set(ins.map(ins => ins.bytes)));
const modes = Array.from(new Set(ins.map(ins => ins.mode)));

type Instruction = {
  hex: string,
  clk: number,
  name: string,
  bytes: number,
}

/*
Modes:

im (Immediate)
M(PC + 1)
#<DATA>

ab (Absolute)
[M(PC + 1)]

pc (Relative)


ih -> Inherent (nothing)
pc -> Relativ (PC + Offset -> PC)
ns -> 
nx
ax
ny
ay
x+
x-
+x
-x
y+
y-
+y
-y

*/

function parseInstruction(ins: Instruction) {
  const expects = [];
  const out = [];

  //----------- START -----------

  expects.push(...ins.name.split(/[\s,]/g))
  out.push(ins.hex);

  switch(ins.mode) {
    case "ih":
      break;
    case "im":
      expects.push('#Data')
      out.push("#Data");
      break;
    case "ab":
      expects.push('AbsAdr')
      out.push('AbsAdr')
      break;
    case "pc":
      expects.push('OffsetAdr')
      out.push('OffsetAdr')
      break;
    case "nx":
      expects.push("n")
      expects.push("X")
      out.push("n")
      break;
    case "ns":
      expects.push("n")
      expects.push("SP")
      out.push("n")
      break;
    case "ax":
      expects.push("A")
      expects.push("X")
      break;
    case "ny":
      expects.push("n")
      expects.push("Y")
      out.push("n")
      break;
    case "ay":
      expects.push('A')
      expects.push('Y')
      break;
    case "x+":
      expects.push('X+')
      break;
    case "x-":
      expects.push('X-')
      break;
    case "+x":
      expects.push('+X')
      break;
    case "-x":
      expects.push('-X')
      break;
    case "y+":
      expects.push('Y+')
      break;
    case "y-":
      expects.push('Y-')
      break;
    case "+y":
      expects.push('+Y')
      break;
    case "-y":
      expects.push('-Y')
      break;
    default:
      throw new Error(`Invalid mode: ${ins.mode}`);
  }

  return { expects, out };
}

const parsedIns = ins.map(parseInstruction)
console.log(parsedIns)

function group(data: Array<{ expects: string[], out: string[] }>, idx = 0) {
  if(idx > data[0].expects) {
    return 123;
  }
  const unique = {};

  for(const ins of data) {
    const id = ins.expects[idx];
    if(unique.hasOwnProperty(id)) {
      unique[id].push(ins)
    } else {
      unique[id] = [ins]
    }
  }

  for(const ins of Object.keys(unique)) {
    const innerData = unique[ins]

    if(innerData.length === 0) {
      throw new Error('no length')
    } else if(innerData.length === 1) {
      unique[ins] = innerData[0];
    } else {
      unique[ins] = group(innerData, idx + 1)
    }
  }

  return unique
}

const tree = group(parsedIns);
console.log(tree);

console.log(`
match self.curr() {
  Token::Directive(_) => todo!(),
  Token::Sym(_) => todo!(),
  Token::Instruction(ins) => {
    self.advance();
    match self.curr() {
${[...printTree(tree, 'Instruction::', '', 3)].join('\n')}
    }
  },
  Token::Number(_) => todo!(),
}
`)

const namedLiterals = `
  SP
  X
  A
  Y
  X+
  X-
  +X
  -X
  Y+
  Y-
  +Y
  -Y`.split(/\s+/g)
  .map(ins => `"${ins}" => NamedLiteral::${ins.replaceAll('+', 'Plus').replaceAll('-', 'Minus')},`)
  .join('\n');

console.log(
    namedLiterals
)

function* printTree(tree, key_st: string, key_end: string, ident: number) {
  const keys = Object.keys(tree);
  const spaces = '  '.repeat(ident);
  for(const key of keys) {
    const val = tree[key];
    if(val.hasOwnProperty('expects')) {
      // Leaf node
      yield (`${spaces}${key_st}${getMatchKey(key)}${key_end} => {
${[].join('\n')}
${spaces}},`
      )
    } else {
      yield (`${spaces}${key_st}${getMatchKey(key)}${key_end} => {
${spaces}  self.advance();
${spaces}  match self.curr() {
${[...printTree(val, '', '', ident + 2)].join('\n')}
${spaces}  },
${[].join('\n')}
${spaces}},`
      )
    }

  }
  yield `${spaces}_ => return Err(()),`;
}

function getMatchKey(key: string): string {
  switch(key) {
    case 'SP':
    case 'X':
    case 'A':
    case 'Y':
    case 'X+':
    case 'X-':
    case '+X':
    case '-X':
    case 'Y+':
    case 'Y-':
    case '+Y':
    case '-Y':
      const val = key.replace('+', 'Plus').replace('-', 'Minus')
      return `NamedLiteral::${val}`
  }


}

/*

Directives:
  ORG <Val>
  Sym EQU <Val>
  [Sym] FCB <Val>,<Val>...
  [Sym] FCS "<ASCII tecken>"
  [Sym] RMB <Val>

Number Literals:
  OffsetAdr -> 250 | $FA | %11111010
  AbsAdr    -> 250 | $FA | %11111010
  n         -> 250 | $FA | %11111010
  #Data     -> #250 | #$FA | #%11111010

    Number prefixes:
      (none) -> decimal
      $ -> hexadecimal
      % -> binary

Named Literals:
  SP
  X
  A
  Y
  X+
  X-
  +X
  -X
  Y+
  Y-
  +Y
  -Y
*/

/*
let instr = self.curr();
self.advance();

let out: Vec<String> = match insr {
  "LDA" => {
    self.advance();
    match self.curr() {
      Ins::Literal(lit) => {}
      Ins::Named(var) => {
        self.advance();
        match var {
          "n" => {
            
          }
        }
      }
      _ => return Err(()),
    }
  }
}
*/







