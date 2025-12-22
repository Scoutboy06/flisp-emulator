import instructions from "../all.json"

const ins = Object.values(instructions)
  .filter(ins => ins.name)

// const hexes = Array.from(new Set(ins.map(ins => ins.hex)));
// const clks = Array.from(new Set(ins.map(ins => ins.clk)));
// const names = Array.from(new Set(ins.map(ins => ins.name)));
// const bytes = Array.from(new Set(ins.map(ins => ins.bytes)));
// const modes = Array.from(new Set(ins.map(ins => ins.mode)));

type Instruction = {
  hex: string,
  clk: number,
  name: string,
  bytes: number,
  mode: string,
}

console.log(
  Array.from(
    new Set(
      ins.map(
        ins =>
          ins.name.split(' ')[0]
      )
    )
  )
    .join('|')
)

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
  const expects: (string | null)[] = [];
  const out: string[] = [];

  //----------- START -----------

  expects.push(...ins.name.split(/[\s,]/g).map(i => i === 'S' ? 'SP' : i === 'C' ? 'CC' : i))
  out.push(ins.hex);

  switch (ins.mode) {
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
      expects.push(null)
      expects.push('X+')
      break;
    case "x-":
      expects.push(null)
      expects.push('X-')
      break;
    case "+x":
      expects.push(null)
      expects.push('+X')
      break;
    case "-x":
      expects.push(null)
      expects.push('-X')
      break;
    case "y+":
      expects.push(null)
      expects.push('Y+')
      break;
    case "y-":
      expects.push(null)
      expects.push('Y-')
      break;
    case "+y":
      expects.push(null)
      expects.push('+Y')
      break;
    case "-y":
      expects.push(null)
      expects.push('-Y')
      break;
    default:
      throw new Error(`Invalid mode: ${ins.mode}`);
  }

  return { expects, out };
}

const parsedIns = ins.map(parseInstruction)
// console.log(parsedIns)

function group(data: Array<{ expects: string[], out: string[] }>, idx = 0) {
  const unique = {};

  for (const ins of data) {
    const id = ins.expects[idx];
    if (unique.hasOwnProperty(id)) {
      unique[id].push(ins)
    } else {
      unique[id] = [ins]
    }
  }

  for (const ins of Object.keys(unique)) {
    const innerData = unique[ins]

    if (innerData.length === 0) {
      throw new Error('no length')
    } else if (innerData.length === 1) {
      unique[ins] = innerData[0];
    } else {
      unique[ins] = group(innerData, idx + 1)
    }
  }

  return unique
}

// console.log(JSON.stringify(parsedIns, null, 2));

// console.log(
//   Array.from(new Set(
//     parsedIns
//       .flatMap(ins => ins.expects.slice(1))
//   ))
// )
// console.log(
//   Array.from(new Set(
//     parsedIns
//       .flatMap(ins => ins.out.slice(1))
//   ))
// )

type InstrSpec = {
  expects: string[];
  out: string[];
};

const regAtoms = new Set([
  "A", "X", "Y", "SP", "CC",
  "X+", "X-", "+X", "-X",
  "Y+", "Y-", "+Y", "-Y",
]);

const namedLiteralMap = {
  "A": "A",
  "X": "X",
  "Y": "Y",
  "SP": "SP",
  "CC": "CC",

  "X+": "XPlus",
  "X-": "XMinus",
  "+X": "PlusX",
  "-X": "MinusX",

  "Y+": "YPlus",
  "Y-": "YMinus",
  "+Y": "PlusY",
  "-Y": "MinusY",
};

function genAtom(name: string | null): string {
  if (name === null) {
    return "Atom::None";
  }

  if (name in namedLiteralMap) {
    return `Atom::Reg(NamedLiteral::${namedLiteralMap[name]})`;
  }

  // numeric / symbolic operand
  return "Atom::Number(n)";
}

const operandOutMap: Record<string, string> = {
  "AbsAdr": "Operand::AbsAdr(n)",
  "OffsetAdr": "Operand::RelAdr(n)",
  "n": "Operand::N(n)",
  "#Data": "Operand::Imm(n)",
};

function genOperandForm(expects: (string | null)[]): string {
  const tail = expects.slice(1);

  if (tail.length === 0) {
    return "OF::None";
  }

  if (tail[0] === "#Data") {
    if (tail.length === 1) {
      return "OF::Imm1(Atom::Number(n))";
    }
    return `OF::Imm2(Atom::Number(n), ${genAtom(tail[1])})`;
  }

  if (tail.length === 1) {
    return `OF::One(${genAtom(tail[0])})`;
  }

  if (tail.length === 2) {
    return `OF::Two(${genAtom(tail[0])}, ${genAtom(tail[1])})`;
  }

  throw new Error(`Unsupported expects: ${expects.join(", ")}`);
}

function genOperandOut(out: string[]): string {
  if (out.length === 1) {
    return "op0(0x" + out[0] + ")";
  }

  const kind = out[1];
  const operand = operandOutMap[kind];
  if (!operand) {
    throw new Error(`Unknown out operand: ${kind}`);
  }

  return `op1(0x${out[0]}, ${operand})`;
}

function genMatchArm(spec: InstrSpec): string {
  const ins = spec.expects[0];
  const form = genOperandForm(spec.expects);
  const out = genOperandOut(spec.out);

  return `
(I::${ins}, ${form}) => Ok(${out}),`.trim();
}



// let inputs = parsedIns.slice(0);
// inputs.sort((a, b) => {
//   const opcodeA = parseInt(a.out[0], 16);
//   const opcodeB = parseInt(b.out[0], 16);
//   return opcodeA - opcodeB;
// });
// const outputs = inputs.map(genMatchArm);
// console.log(outputs.join("\n"));


// const tree = group(parsedIns);




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







