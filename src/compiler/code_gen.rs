use crate::compiler::parser::{AssignmentOpType, NodeType, ScopeType, TokenNode};
use std::collections::HashMap;

use super::lexer::RhTypes;

// This is technically somehow working right now and I have no idea why, I think it does things
// backwards but it's ok
/// This struct manages the stack and all scopes
/// It stores the furthest offset from the base of the stack
/// This model works on the assumption that the base of the stack shifts around as well
/// So a store looks like this
/// "store x1, [sp, #{}]", (furthest_offset + sp_offset)

/// Reusable code snippets
// const NEW_SCOPE_BYTE_ALIGN: &str = "\n.balign 4\n.L{}:";

const _MOV_EXPR_1: &str = "\nldr x9 [sp], #8";
const _MOV_EXPR_2: &str = "\nldr x10 [sp], #8";
const _ADD_EXPR: &str = "\nadd x9, x9, x10";

// const GET_ADR: &str = "\nldr {} [sp], #16"; // assuming it was 32 bit address formatting needed
// const STR_ADR: &str = "\nstr {}, [sp, #-16]!";

const _CMP_EXPR: &str = "\ncmp x9, x0"; // comparison flag set in the processor status register
                                        // const SYS_CALL_NUM: &str = "\nmov x0, {}"; // syscall_number
                                        // const SYS_CALL_ARG1: &str = "\nmov x0, {}"; // etc
const _SYS_CALL: &str = "\nsvc #0x80";
#[derive(Debug, Clone)]
pub struct FunctionSig {
    args: Vec<(String, i32)>, // ids in order
    label: String,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    table: HashMap<String, i32>, // id, offsert from current stack frame base
    function_table: HashMap<String, FunctionSig>, // function name, label
    parent: Option<usize>,       // the index in the
    furthest_offset: i32,
}

impl SymbolTable {
    fn new(parent: Option<usize>) -> SymbolTable {
        SymbolTable {
            table: HashMap::new(),
            function_table: HashMap::new(),
            parent,
            furthest_offset: 0,
        }
    }

    fn get_function(&self, id: String) -> Option<&FunctionSig> {
        self.function_table.get(&id).to_owned()
    }

    fn get_id(&self, id: String) -> Option<&i32> {
        self.table.get(&id).to_owned()
    }

    fn new_id(&mut self, id: String, size: i32) {
        self.table.insert(id, -self.furthest_offset);

        self.furthest_offset += size;
    }
}

pub struct Handler {
    scopes: Vec<String>,
    curr_scope: usize,
    curr_frame: usize,
    sym_arena: Vec<SymbolTable>,
    break_anchors: Vec<usize>, // each number represents the insertuction pointer that can be broken to from the current scope
}

impl Handler {
    fn new() -> Self {
        let mut handler = Handler {
            scopes: vec![String::from("\n.global .main\n.align 4\n")],
            curr_scope: 0,
            break_anchors: vec![],
            sym_arena: vec![SymbolTable::new(None)],
            curr_frame: 0,
        };

        handler.curr_scope = handler.scopes.len();
        handler.scopes.push(String::new());

        handler
            .push_to_scope("\n.main:\n; x29 is our sfb\n;x15 is our sp\nmov x29, sp\nmov x15, sp");

        handler
    }

    /// Scopes

    fn new_scope(&mut self) {
        self.curr_scope = self.scopes.len();
        self.scopes.push(format!("\n.L{}:", self.curr_scope));
    }

    #[allow(dead_code)]
    fn prev_scope(&mut self) {
        self.curr_scope -= 1;
    }

    fn push_to_scope(&mut self, string: impl ToString) {
        self.scopes[self.curr_scope].push_str(string.to_string().as_str())
    }

    pub fn format_scopes(&self) -> String {
        let mut ret = String::new();

        let mut lines = self.scopes[0].lines();
        ret.push_str(format!("{}\n", lines.next().unwrap()).as_str());
        ret.push_str(format!("{}\n", lines.next().unwrap()).as_str());
        ret.push_str(format!("{}\n", lines.next().unwrap()).as_str());

        for scope in 1..self.scopes.len() {
            let mut lines = self.scopes[scope].lines();
            ret.push_str(format!("{}\n", lines.next().unwrap()).as_str());
            ret.push_str(format!("{}\n", lines.next().unwrap()).as_str());
            //ret.push_str(format!("{}\n", lines.next().unwrap()).as_str());

            for line in lines {
                ret.push_str(format!("\t{}\n", line).as_str());
            }
        }

        ret.trim().to_string()
    }
    #[allow(dead_code)]
    pub fn print_scopes(&self) {
        for scope in self.scopes.iter() {
            let mut chars = scope.chars();
            chars.next();
            let mod_scope = chars.as_str().to_string();
            let mut lines = mod_scope.split("\n").collect::<Vec<&str>>().into_iter();
            println!("{}", lines.next().expect("No lines in iterator"));
            for line in lines {
                println!("\t{}", line);
            }
        }
    }

    /// Breaks

    fn insert_break(&mut self) {
        self.push_to_scope(format!("\n; break statement\nb .L{}", self.curr_scope + 1));
        self.new_scope();
    }

    fn new_break_anchor(&mut self) {
        self.break_anchors.push(self.curr_scope);
    }

    /// Frames + Symbol Tables

    /// This function must be breaking some borrow checker rule
    fn new_stack_frame(&mut self) {
        self.sym_arena.push(SymbolTable::new(Some(self.curr_frame)));
        self.curr_frame = self.sym_arena.len() - 1;
    }

    // Panics if id doesn't exist
    fn get_id(&self, id: impl ToString) -> Option<&i32> {
        let sym_res = self.sym_arena[self.curr_frame].get_id(id.to_string());

        if sym_res.is_none() {
            return match &self.sym_arena[self.curr_frame].parent {
                Some(parent) => Some(
                    self.sym_arena[*parent]
                        .get_id(id.to_string())
                        .expect("Symbol not found"),
                ),
                None => return None,
            };
        }

        sym_res
    }

    fn get_function(&self, id: impl ToString) -> Option<&FunctionSig> {
        let sym_res = self.sym_arena[self.curr_frame].get_function(id.to_string());
        if sym_res.is_none() {
            return match &self.sym_arena[self.curr_frame].parent {
                Some(parent) => Some(
                    self.sym_arena[*parent]
                        .get_function(id.to_string())
                        .expect("Symbol not found"),
                ),
                None => panic!("Symbol not found"),
            };
        }

        sym_res
    }

    fn new_id(&mut self, id: impl ToString, size: i32) {
        self.sym_arena[self.curr_frame].new_id(id.to_string(), size);
    }

    fn new_8_byte(&mut self, id: impl ToString) {
        self.sym_arena[self.curr_frame].new_id(id.to_string(), 8);
    }

    fn new_expr_lit(&mut self) {
        self.sym_arena[self.curr_frame].furthest_offset += 8;
    }

    fn unload_expr_lit(&mut self) {
        self.sym_arena[self.curr_frame].furthest_offset -= 8;
    }

    fn new_function(&mut self, name: impl ToString, scope: i32, args: Vec<(String, i32)>) {
        let new_scope_label = format!(".L{}", scope);
        let function_sig = FunctionSig {
            args: args.clone(),
            label: new_scope_label, // figure out how to determine label (probably based on a property in handler)
        };
        self.sym_arena[self.curr_frame]
            .function_table
            .insert(name.to_string(), function_sig);

        self.new_stack_frame();

        self.sym_arena[self.curr_frame].furthest_offset += 8;
        for arg in args.into_iter() {
            self.new_id(arg.0, arg.1);
        }
    }
}

// Ask Andrew how to enter into main after trying to do it with code_gen, not parsing
pub fn main(node: &TokenNode) -> String {
    let mut handler = Handler::new();
    // use this later
    // var_name : pos_on_stack
    // Stores the variable name and their absolute position on the stack

    // Stores the variable name and their relative position on the stack(from the top of the stack
    // at the begining of runtime). This should be on a per_scope level
    if node.token == NodeType::Program {
        scope_code_gen(
            &node.children.as_ref().unwrap()[0],
            &mut handler,
            ScopeType::Program,
        );
    }
    handler.push_to_scope("\n\n; exit program gracefully\nmov x0, #0\nmov x16, #1\nsvc #0x80");

    // code.trim().to_string()

    handler.format_scopes()
}

pub fn scope_code_gen(node: &TokenNode, handler: &mut Handler, scope_type: ScopeType) {
    for child_node in node.children.as_ref().expect("Scope to have children") {
        match &child_node.token {
            NodeType::Declaration(arg) => {
                declare_code_gen(&child_node, handler, arg.0.clone(), arg.1.clone());
            }
            NodeType::Assignment(_) => {
                assignment_code_gen(&child_node, handler);
            }
            NodeType::If => {
                if_code_gen(&child_node, handler);
            }
            NodeType::While => {
                while_code_gen(&child_node, handler);
            }
            NodeType::_Loop => {}
            NodeType::FunctionCall(id) => {
                function_call_code_gen(&child_node, handler, id.to_string())
            }
            NodeType::FunctionDecaration((id, t)) => {
                function_declare_code_gen(&child_node, handler, id.to_string(), t.clone())
            }
            NodeType::Break => handler.insert_break(),
            NodeType::Return => {
                if let ScopeType::Function(_t) = scope_type {
                    // return type shouldn't matter for now
                    return return_statement_code_gen(&child_node, handler);
                } else {
                    panic!("Return statements only allowed in function scopes");
                }
            }
            NodeType::Asm(str) => asm_code_gen(&child_node, handler, str.to_string()),
            NodeType::PutChar => putchar_code_gen(&child_node, handler),
            NodeType::Assert => assert_code_gen(&child_node, handler),
            _ => {}
        };
    }
    if scope_type == ScopeType::While {
        handler.push_to_scope(format!(
            "\n; break statement\nb .L{}",
            handler.curr_scope + 1
        ));
    }
}

/// Returns the generated code
/// Modifies the sp
pub fn declare_code_gen(node: &TokenNode, handler: &mut Handler, name: String, _t: RhTypes) {
    handler.push_to_scope(format!(
        "\n\n; var dec: {}, offset: {} (wrong for arrays)",
        name,
        handler.sym_arena[handler.curr_frame].furthest_offset + 8
    ));
    expr_code_gen(
        &node.children.as_ref().expect("Node to have children")[0],
        handler,
    );

    handler.new_id(&name, 0);
    handler.push_to_scope("\n");
}

pub fn assignment_code_gen(node: &TokenNode, handler: &mut Handler) {
    handler.push_to_scope("\n; variable assignment");
    println!("Assignment Node: {:?}", node.token);
    let children = &node
        .children
        .as_ref()
        .expect("Assignment token need children");
    assert_eq!(children.len(), 2);

    let op = match &node.token {
        NodeType::Assignment(op) => op,
        _ => panic!("Given given invalid assignment node"),
    };
    let relative_stack_position: String = match &children[0].token {
        NodeType::Id(name) => {
            let pos = handler.get_id(name).expect("Undefined Identifier").clone();
            format!("#{}", pos).to_string()
        }
        NodeType::DeRef => {
            println!("node:\n{:?}", children[0]);
            handler.push_to_scope("\n\n; deref assignment");
            expr_code_gen(
                &children[0]
                    .children
                    .as_ref()
                    .expect("deref must have children in assignment")[0],
                handler,
            );
            handler.push_to_scope("\n\n; str offset to assign to\nmov x12, x15");
            format!("x12").to_string()
        }
        _ => panic!("Only Ids are legal for assignments"),
    };
    // TODO: Figure out how to handle expressions

    expr_code_gen(&node.children.as_ref().unwrap()[1], handler);
    handler.push_to_scope("\nldr x9, [x15], #8");
    handler.unload_expr_lit();

    // this load the old value in case we need it
    if *op != AssignmentOpType::Eq {
        if relative_stack_position == "x12" {
            handler.push_to_scope("\nsub x12, x29, x12\nldr x10, [x13]");
        } else {
            handler.push_to_scope(format!("\nldr x10, [x29, {relative_stack_position}]"));
        }
        match op {
            AssignmentOpType::AddEq => handler.push_to_scope("\nadd x9, x9, x10"),
            AssignmentOpType::SubEq => handler.push_to_scope("\nsub x9, x9, x10"),
            AssignmentOpType::MulEq => handler.push_to_scope("\nmul x9, x9, x10"),
            AssignmentOpType::DivEq => handler.push_to_scope("\nudiv x9, x9, x10"),
            AssignmentOpType::BOrEq => handler.push_to_scope("\nbor x9, x9, x10"),
            AssignmentOpType::BAndEq => handler.push_to_scope("\nand x9, x9, x10"),
            AssignmentOpType::BXorEq => handler.push_to_scope("\nxor x9, x9, x10"),
            _ => panic!("Unexpected Operator"),
        };
    }

    if relative_stack_position == "x12" {
        if *op == AssignmentOpType::Eq {
            handler.push_to_scope("\nsub x12, x29, x12")
        }
        handler.push_to_scope("\nstr x9, [x13]\n");
    } else {
        handler.push_to_scope(format!("\nstr x9, [x29, {relative_stack_position}]\n"));
    }

    //handler.push_to_scope(format!("\nstr x9, [x29], #{relative_stack_position}"));
}

// Leaves the result on TOS
fn expr_code_gen(node: &TokenNode, handler: &mut Handler) {
    match &node.token {
        NodeType::NumLiteral(val) => {
            handler.push_to_scope(format!("\nmov x9, #{val}"));
            handler.push_to_scope("\nstr x9, [x15, #-8]!");
            handler.new_expr_lit();
        }
        NodeType::Id(name) => {
            let offset = handler.get_id(name).expect("Undefined identifier");
            handler.push_to_scope(format!("\nldr x9, [x29, #{offset}]\nstr x9, [x15, #-8]!",));
            handler.new_expr_lit();
        }
        NodeType::FunctionCall(name) => {
            function_call_code_gen(&node, handler, name.to_string());
            handler.push_to_scope("\n; assume ret is TOS");
            handler.new_expr_lit();
        }
        NodeType::Adr(id) => {
            let relative_offset = handler.get_id(id).unwrap().abs();
            println!("{:?}", handler.sym_arena[handler.curr_frame]);

            handler.push_to_scope(format!("\n; getting the adr of: {id}\nmov x9, x29\nmov x10, #{relative_offset}\nsub x9, x9, x10\nstr x9, [x15, #-8]!"));
            handler.new_expr_lit();
        }
        NodeType::DeRef => {
            deref_code_gen(&node, handler);
        }
        NodeType::Array => {
            let children = node
                .children
                .as_ref()
                .expect("Array node should have children");
            handler.push_to_scope("\n; new array\nmov x11, x15 ; anchor ptr");
            for node in children.iter() {
                expr_code_gen(&node, handler);
                handler.push_to_scope("\n")
            }
            handler.push_to_scope("\nstr x11, [x15, #-8]! ; str array anchor TOS");
            handler.new_expr_lit();
        }
        _ => {
            expr_code_gen(&node.children.as_ref().unwrap()[0], handler);
            expr_code_gen(&node.children.as_ref().unwrap()[1], handler);
            handler.push_to_scope("\n\n; load from stack\nldr x10, [x15], #8\nldr x9, [x15], #8");
            handler.unload_expr_lit();
            handler.unload_expr_lit();
            match &node.token {
                NodeType::Add => handler.push_to_scope("\nadd x9, x9, x10"),
                NodeType::Sub => handler.push_to_scope("\nsub x9, x9, x10"),
                NodeType::Div => handler.push_to_scope("\nudiv x9, x9, x10"),
                NodeType::Mul => handler.push_to_scope("\nmul x9, x9, x10"),
                NodeType::MNeg => handler.push_to_scope("\nmneg x9, x9, x10"),
                NodeType::BAnd => handler.push_to_scope("\nand x9, x9, x10"),
                NodeType::BOr => handler.push_to_scope("\nor x9, x9, x10"),
                NodeType::BXor => handler.push_to_scope("\nxor x9, x9, x10"),
                _ => panic!("Expected Expression"),
            };
            handler.push_to_scope("\nstr x9, [x15, #-8]!");
            handler.new_expr_lit();
        }
    }
}

pub fn function_declare_code_gen(
    node: &TokenNode,
    handler: &mut Handler,
    name: String,
    t: RhTypes,
) {
    let children = node
        .children
        .as_ref()
        .expect("Function node must have children");
    let orig_scope = handler.curr_scope;
    let orig_frame = handler.curr_frame;
    handler.new_scope();
    handler.push_to_scope("\n; function declaration");
    let function_scope = handler.curr_scope;
    let mut args: Vec<(String, i32)> = vec![];

    for child in 0..children.len() - 1 {
        if let NodeType::Declaration((id, t)) = &children[child].token {
            let size = match t {
                RhTypes::Char => 8,
                RhTypes::Int => 8,
                RhTypes::Void => 8,
            };
            // TODO: figure out what other code needs to go here (id any)
            args.push((id.clone(), size));
        }
    }
    handler.new_function(name.clone(), function_scope as i32, args);
    let scope_child = &children[children.len() - 1];

    if let NodeType::Scope(_) = scope_child.token {
        scope_code_gen(&scope_child, handler, ScopeType::Function(t.clone()));
    }

    if t == RhTypes::Void {
        handler.push_to_scope(
            "\n\n; unload stack\nmov x15, x29\nadd x15, x15, #8\nldr x29, [x29]\nret",
        );
    }

    handler.curr_scope = orig_scope;
    handler.curr_frame = orig_frame;
}

pub fn function_call_code_gen(node: &TokenNode, handler: &mut Handler, name: String) {
    // This assembly should:
    // Store the address of the current stack fb on the top of the stack
    // Decrement the sp by 32
    // load the address of the new stack frame base into the sfb register
    handler.push_to_scope("\n\n; place old sfb\nstr x29, [x15, #-8]!\nmov x10, x15");

    handler.new_expr_lit();

    let children = node.children.as_ref().expect("Function has no children");

    let function_sig = handler
        .get_function(&name)
        .expect("Function id not found")
        .clone();

    // TODO: Check if the return address (pc) needs to go on the stack
    // Place the return label on the stack
    //handler.push_to_scope(format!(
    //    "\nmov x9, #{}\nstr x9, [x15, #-8]!",
    //    handler.curr_scope
    //));
    for i in 0..function_sig.args.len() {
        expr_code_gen(&children[i], handler);
    }

    handler.push_to_scope("\nmov x29, x10");
    handler.push_to_scope(format!("\nbl {}", function_sig.label));
}

pub fn if_code_gen(node: &TokenNode, handler: &mut Handler) {
    handler.push_to_scope("\n\n; if statement");
    match &node.children {
        Some(children) => {
            // cmp x1, #n
            // beq label
            // node.children.unwrap()[0] is condition node, other child is scope

            condition_expr_code_gen(&children[0], handler);
            handler.push_to_scope("\n; scope of if statement");
            scope_code_gen(&children[1], handler, ScopeType::If);
            handler.push_to_scope("\nret");
            handler.curr_scope -= 1;
        }
        None => {
            panic!("Expected Condition");
        }
    };
}

pub fn deref_code_gen(node: &TokenNode, handler: &mut Handler) {
    let children = node
        .children
        .as_ref()
        .expect("DeRef node should have children");
    assert_eq!(children.len(), 1);
    expr_code_gen(&children[0], handler);
    handler.push_to_scope(
        "\n\n; deref expr\nldr x9, [x15], #8\nsub x9, x9, #8\nldr x10, [x9]\nstr x10, [x15, #-8]!",
    );
}

pub fn condition_expr_code_gen(node: &TokenNode, handler: &mut Handler) {
    let children: Vec<TokenNode> = node.children.as_ref().unwrap_or(&Vec::new()).to_vec();
    println!("condition token: {:?}", node.token);
    match &node.token {
        NodeType::AndCmp => {
            condition_expr_code_gen(&children[0], handler);
            condition_expr_code_gen(&children[1], handler);
        }
        // This makes putting booleans into conditions illegal
        // TODO: Fix this or add it to parsing restraints
        NodeType::OrCmp => {
            condition_expr_code_gen(&children[0], handler);
            handler.curr_scope -= 1;
            handler.scopes.pop();
            condition_expr_code_gen(&children[1], handler);
        }
        NodeType::NeqCmp => {
            condition_expr_code_gen(&children[0], handler);
            condition_expr_code_gen(&children[1], handler);

            handler.push_to_scope(format!(
                "\nldr x9, [x15], #8\nldr x10, [x15], #8\ncmp x9, x10\nbne .L{}",
                handler.curr_scope + 1
            ));
            handler.new_scope();
        }
        NodeType::EqCmp => {
            condition_expr_code_gen(&children[0], handler);
            condition_expr_code_gen(&children[1], handler);

            handler.push_to_scope(format!(
                "\nldr x9, [x15], #8\nldr x10, [x15], #8\ncmp x9, x10\nbeq .L{}",
                handler.curr_scope + 1
            ));
            handler.unload_expr_lit();
            handler.unload_expr_lit();
            handler.new_scope();
        }
        _ => {
            expr_code_gen(&node, handler);
        }
    };
}

fn while_code_gen(node: &TokenNode, handler: &mut Handler) {
    match &node.children {
        Some(children) => {
            handler.push_to_scope(format!("\nb .L{}", handler.curr_scope + 1));
            handler.new_break_anchor();
            handler.new_scope();

            condition_expr_code_gen(&children[0], handler);
            scope_code_gen(&children[1], handler, ScopeType::While);
            handler.new_scope();
        }
        None => {
            panic!("Expected Condition")
        }
    }
}

fn assert_code_gen(node: &TokenNode, handler: &mut Handler) {
    // Exit the program with error
    let children = node.children.as_ref().expect("Assert missing children");
    if children.len() != 1 {
        panic!("Assert incorrect children: {}", children.len());
    }
    condition_expr_code_gen(&children[0], handler);
    handler.push_to_scope("\nmov x0, #1\nmov x8, #93\nsvc #0")
}

fn return_statement_code_gen(node: &TokenNode, handler: &mut Handler) {
    handler.push_to_scope("\n\n; evaluate return statement and place on stack");
    expr_code_gen(
        &node
            .children
            .as_ref()
            .expect("Return statement has no expression")[0],
        handler,
    );
    handler.push_to_scope("\n\n; ldr expr into x9\nldr x9, [x15], #8");
    handler.push_to_scope("\n\n; reset sfb\nmov x15, x29\nadd x15, x15, #8\nldr x29, [x29]");
    handler.push_to_scope("\nstr x9, [x15, #-8]!\nret");
    //handler.curr_frame = handler.sym_arena[handler.curr_frame]
    //    .parent
    //    .expect("Functions must have parent scopes");
}

fn asm_code_gen(_node: &TokenNode, handler: &mut Handler, str: String) {
    handler.push_to_scope(format!("\n{}", str));
}

fn putchar_code_gen(node: &TokenNode, handler: &mut Handler) {
    let children = node
        .children
        .as_ref()
        .expect("Putchar node has no children");

    expr_code_gen(&children[0], handler);
    // We can assume that the result goes on top of the stack
    handler.push_to_scope("\n\n; putchar\nmov x0, #1 ; stdout\nmov x1, x15 ; put from TOS\nmov x2, #1 ; print 1 char\nmov x16, #4 ; write\nsvc #0x80\n; unload the TOS\nadd x15, x15, #8\n");
    handler.unload_expr_lit();
}
