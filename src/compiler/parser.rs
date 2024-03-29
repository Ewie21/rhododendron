use crate::compiler::error::{RhErr, ET};
use crate::compiler::lexer::{LineNumHandler, RhTypes, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum ScopeType {
    Function,
    While,
    Program,
    If,
    Loop,
    For,
}

// Valid Node Types
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Program,
    Sub,
    Div,
    Eq,
    Id(String), // figure out if we want this here
    EqCmp,
    NeqCmp,
    BOr,
    BAnd,
    BXor,
    BOrEq,
    BAndEq,
    BXorEq,
    SubEq,
    AddEq,
    DivEq,
    MulEq,
    Mul,
    AndCmp,
    OrCmp,
    NumLiteral(i32),
    Add,
    If,
    For,
    While,
    Loop,
    Break,
    FunctionCall(String),
    Scope(Option<RhTypes>), // <-- anything that has {} is a scope, scope is how we're handling multiple statements, scopes return the last statement's result or void
    Condition(bool), // true is eq false is neq; This might not be completely clear when optimizing conditionals and loops start
    Assignment(String),
    Declaration((String, RhTypes)),
    Asm(String),
    FunctionDecaration(String),
    Type(RhTypes),
}

impl NodeType {
    fn from_token(tok: Token) -> Result<NodeType, ()> {
        match tok {
            Token::Sub => Ok(NodeType::Sub),
            Token::Div => Ok(NodeType::Div),
            Token::Eq => Ok(NodeType::Eq),
            Token::Id(str) => Ok(NodeType::Id(str.to_string())),
            Token::EqCmp => Ok(NodeType::EqCmp),
            Token::NeqCmp => Ok(NodeType::NeqCmp),
            Token::BOr => Ok(NodeType::BOr),
            Token::BAnd => Ok(NodeType::BAnd),
            Token::BXor => Ok(NodeType::BXor),
            Token::BOrEq => Ok(NodeType::BOrEq),
            Token::BAndEq => Ok(NodeType::BAndEq),
            Token::BXorEq => Ok(NodeType::BXorEq),
            Token::SubEq => Ok(NodeType::SubEq),
            Token::AddEq => Ok(NodeType::AddEq),
            Token::DivEq => Ok(NodeType::DivEq),
            Token::MulEq => Ok(NodeType::MulEq),
            Token::Star => Ok(NodeType::Mul), // exception for pointer
            Token::NumLiteral(i) => Ok(NodeType::NumLiteral(i)),
            Token::Add => Ok(NodeType::Add),
            Token::For => Ok(NodeType::For),
            Token::While => Ok(NodeType::While),
            Token::If => Ok(NodeType::If),
            Token::Break => Ok(NodeType::Break),
            _ => {
                println!("Oh God No, Not A Valid Token");
                return Err(());
            }
        }
    }
}

pub struct TokenHandler {
    tokens: Vec<Token>,
    curr_token: usize,
    token_lines: Vec<i32>,
}

impl TokenHandler {
    pub fn new(tokens: Vec<Token>, line_tracker: LineNumHandler) -> Self {
        TokenHandler {
            tokens,
            curr_token: 0,
            token_lines: line_tracker.token_lines,
        }
    }

    pub fn next_token(&mut self) {
        self.curr_token += 1;
    }

    pub fn peek(&self, i: usize) -> Token {
        self.tokens[self.curr_token + i].clone()
    }

    pub fn prev_token(&mut self) {
        self.curr_token -= 1;
    }

    pub fn get_token(&self) -> &Token {
        &self.tokens[self.curr_token]
    }

    pub fn get_prev_token(&self) -> &Token {
        &self.tokens[self.curr_token - 1]
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn new_err(&self, err: ET) -> RhErr {
        RhErr {
            err,
            line: self.token_lines[self.curr_token],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenNode {
    pub token: NodeType,
    pub children: Option<Vec<TokenNode>>,
}

impl std::fmt::Display for TokenNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.token) // doesn't print values
    }
}

impl TokenNode {
    pub fn new(token: NodeType, children: Option<Vec<TokenNode>>) -> TokenNode {
        TokenNode { token, children }
    }

    pub fn print(&self, n: &mut i32) {
        (0..*n).into_iter().for_each(|_| {
            print!("    ");
        });
        println!("{}", self);
        *n += 1;
        if let Some(children) = &self.children {
            children.iter().for_each(|node| {
                node.print(n);
            })
        }
        *n -= 1;
        // println!("End Children");
    }
}

pub fn program(tokens: Vec<Token>, line_tracker: LineNumHandler) -> Result<TokenNode, RhErr> {
    let mut token_handler = TokenHandler::new(tokens, line_tracker);

    let mut program_node = TokenNode::new(NodeType::Program, Some(vec![]));
    let top_scope = scope(&mut token_handler, ScopeType::Program)?;
    program_node.children.as_mut().unwrap().push(top_scope);

    program_node.print(&mut 0);
    println!("past parsing");
    Ok(program_node)
}

pub fn scope(token_handler: &mut TokenHandler, scope_type: ScopeType) -> Result<TokenNode, RhErr> {
    let mut scope_node = TokenNode::new(NodeType::Scope(None), Some(vec![]));
    while *token_handler.get_token() != Token::CCurl {
        if token_handler.curr_token > token_handler.len() {
            return Err(token_handler.new_err(ET::ExpectedCParen));
        }

        scope_node
            .children
            .as_mut()
            .expect("Scope has no children")
            .push(statement(token_handler, scope_type.clone())?);
        println!();
        if token_handler.curr_token == token_handler.len() - 1 {
            return Ok(scope_node);
        }
        token_handler.next_token();
        // println!("here\n");
        // if token_handler.len() == token_handler.curr_token + 1 {
        // if *token_handler.get_token() != Token::Semi {
        // scope_node.token = NodeType::Scope(Some(RhTypes::Int)) // TODO: Chane this to evaluate the type of the last statement
        // }
        // if *token_handler.get_token() == Token::CCurl { break; }
        // }
    }
    if *token_handler.get_prev_token() == Token::Semi {
        scope_node.token = NodeType::Scope(Some(RhTypes::Int)) // TODO: Change this to evaluate the  type of the last statement
    }
    println!("past scope\n");
    Ok(scope_node)
}

pub fn statement(
    token_handler: &mut TokenHandler,
    scope_type: ScopeType,
) -> Result<TokenNode, RhErr> {
    // let mut node: TokenNode = TokenNode::new(NodeType::Program, Some(vec![])); // todo: add default type
    let statement_token = token_handler.get_token();
    println!("Statement Token: {:?}", statement_token);
    match statement_token {
        Token::Type(t) => type_statement(token_handler, t.clone()),
        Token::Id(name) => id_statement(token_handler, name.to_string()),
        Token::If => if_statement(token_handler),
        Token::While => while_statement(token_handler),
        Token::For => for_statement(token_handler),
        Token::Break => {
            if scope_type == ScopeType::While || scope_type == ScopeType::Loop {
                Ok(TokenNode::new(NodeType::Break, None))
            } else {
                Err(token_handler.new_err(ET::ExpectedStatement))
            }
        }
        Token::Asm => asm_statement(token_handler),
        _ => Err(token_handler.new_err(ET::ExpectedStatement)),
    }
}

fn declaration(token_handler: &mut TokenHandler, t: RhTypes) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    match token_handler.get_token() {
        Token::Id(id) => {
            let mut node = TokenNode::new(NodeType::Declaration((id.to_string(), t)), Some(vec![]));
            token_handler.next_token();
            if *token_handler.get_token() == Token::Eq {
                token_handler.next_token();
                node.children
                    .as_mut()
                    .expect("node to have children")
                    .push(expression(token_handler)?);
            }
            Ok(node.clone())
        }
        _ => Err(token_handler.new_err(ET::ExpectedId)),
    }
}

fn declaration_statement(token_handler: &mut TokenHandler, t: RhTypes) -> Result<TokenNode, RhErr> {
    let declare = declaration(token_handler, t);
    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }
    declare
}

fn expression(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let mut left = term(token_handler)?;
    if token_handler.peek(1) != Token::Add && token_handler.peek(1) != Token::Sub {
        return Ok(left);
    }
    token_handler.next_token();
    let mut curr = token_handler.get_token();
    println!("Expression curr: {:?}", curr);
    while *curr == Token::Add || *curr == Token::Sub {
        let op = curr.clone();
        let right = term(token_handler)?;
        let op_tok = TokenNode::new(NodeType::from_token(op).unwrap(), Some(vec![left, right]));

        left = op_tok;
        curr = &token_handler.get_token();
    }
    Ok(left)
}

fn term(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let mut left: TokenNode = factor(token_handler)?;
    if token_handler.peek(1) != Token::Div && token_handler.peek(1) != Token::Star {
        return Ok(left);
    }
    token_handler.next_token();
    let mut curr = token_handler.get_token();
    println!("Term curr: {:?}", curr);
    while *curr == Token::Star || *curr == Token::Div {
        let op = curr.clone();
        token_handler.next_token();
        let right = factor(token_handler)?;
        let op_tok = TokenNode::new(NodeType::from_token(op).unwrap(), Some(vec![left, right]));
        left = op_tok;
        curr = &token_handler.get_token();
    }
    Ok(left)
}

fn factor(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let token = token_handler.get_token();
    println!("Factor: {:?}, ", token);
    match token {
        Token::NumLiteral(num) => Ok(TokenNode::new(NodeType::NumLiteral(*num), None)),
        Token::Id(id) => Ok(TokenNode::new(NodeType::Id(id.to_string()), None)),
        Token::OParen => {
            token_handler.next_token();
            match expression(token_handler) {
                Ok(node) => {
                    if *token_handler.get_token() == Token::CParen {
                        Ok(node)
                    } else {
                        Err(token_handler.new_err(ET::ExpectedCParen))
                    }
                }
                Err(err) => Err(err),
            }
        }
        _ => Err(token_handler.new_err(ET::ExpectedExpression)),
    }
}

fn assignment(token_handler: &mut TokenHandler, name: String) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    let token = TokenNode::new(
        NodeType::Assignment(name),
        Some(vec![
            TokenNode::new(
                NodeType::from_token(token_handler.get_token().clone()).expect("valid id"),
                None,
            ),
            expression(token_handler)?,
        ]),
    );
    token_handler.next_token();
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }

    Ok(token)
}

fn while_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let mut while_node = TokenNode::new(NodeType::While, Some(vec![]));
    token_handler.next_token();
    let condition_node = condition(token_handler)?;
    while_node
        .children
        .as_mut()
        .expect("While children to be some")
        .push(condition_node);

    token_handler.next_token();
    token_handler.next_token();

    let scope_node = scope(token_handler, ScopeType::While)?;
    while_node
        .children
        .as_mut()
        .expect("While children to be ssome")
        .push(scope_node);
    Ok(while_node)
}

fn if_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let mut if_node = TokenNode::new(NodeType::If, Some(vec![]));
    token_handler.next_token(); // might make semi handled by the called functions instead
    let condition_node = condition(token_handler)?;
    if_node
        .children
        .as_mut()
        .expect("If children to be some")
        .push(condition_node);

    token_handler.next_token();
    token_handler.next_token();

    let scope_node = scope(token_handler, ScopeType::If)?;
    if_node
        .children
        .as_mut()
        .expect("children to be some")
        .push(scope_node);
    Ok(if_node)
}

fn function_declare_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    println!("Function Declaration");
    token_handler.next_token();
    let token = token_handler.get_token();
    println!("Token: {:?}", token);
    if let Token::Id(id) = token {
        let mut function_node =
            TokenNode::new(NodeType::FunctionDecaration(id.clone()), Some(vec![]));
        token_handler.next_token();
        if *token_handler.get_token() != Token::OParen {
            return Err(token_handler.new_err(ET::ExpectedCParen));
        }
        token_handler.next_token();
        loop {
            let t = match token_handler.get_token() {
                Token::Type(t) => t,
                _ => return Err(token_handler.new_err(ET::ExpectedType)),
            };
            let declaration_node = declaration(token_handler, t.clone())?;
            function_node
                .children
                .as_mut()
                .unwrap()
                .push(declaration_node);
            // token_handler.next_token();
            if *token_handler.get_token() != Token::Comma {
                break;
            }
        }
        // token_handler.next_token();
        println!("Cparent: {:?}", token_handler.get_token());
        if *token_handler.get_token() != Token::CParen {
            return Err(token_handler.new_err(ET::ExpectedCParen));
        }
        token_handler.next_token();
        if *token_handler.get_token() == Token::Arrow {
            token_handler.next_token();
            if let Token::Type(t) = token_handler.get_token() {
                function_node
                    .children
                    .unwrap()
                    .push(TokenNode::new(NodeType::Type(t.clone()), None));
            }
            return Err(token_handler.new_err(ET::ExpectedType));
        }
        println!("Pre Scope Token: {:?}", token_handler.get_token());
        token_handler.next_token();
        let scope_node = scope(token_handler, ScopeType::Function)?;
        function_node.children.as_mut().unwrap().push(scope_node);

        return Ok(function_node);
    }

    Err(token_handler.new_err(ET::ExpectedId))
}

fn function_call_statement(
    token_handler: &mut TokenHandler,
    name: String,
) -> Result<TokenNode, RhErr> {
    let call_node = function_call(token_handler, name)?;
    token_handler.next_token();
    println!("post call statement {:?}", token_handler.get_token());
    if *token_handler.get_token() != Token::Semi {
        return Err(token_handler.new_err(ET::ExpectedSemi));
    }
    Ok(call_node)
}

fn function_call(token_handler: &mut TokenHandler, name: String) -> Result<TokenNode, RhErr> {
    let mut function_call_node = TokenNode::new(NodeType::FunctionCall(name), Some(vec![]));
    token_handler.next_token();
    loop {
        println!("Call arg: {:?}", token_handler.get_token());
        let arg_node = expression(token_handler)?;
        function_call_node.children.as_mut().unwrap().push(arg_node);
        token_handler.next_token();
        if *token_handler.get_token() != Token::Comma {
            break;
        }
    }
    println!("{:?}", token_handler.get_token());
    if *token_handler.get_token() != Token::CParen {
        return Err(token_handler.new_err(ET::ExpectedCParen));
    }
    Ok(function_call_node)
}

fn id_statement(token_handler: &mut TokenHandler, id: String) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    match token_handler.get_token() {
        Token::OParen => function_call_statement(token_handler, id),
        _ => assignment(token_handler, id),
    }
}

fn type_statement(token_handler: &mut TokenHandler, t: RhTypes) -> Result<TokenNode, RhErr> {
    match token_handler.peek(2) {
        Token::OParen => function_declare_statement(token_handler),
        _ => declaration_statement(token_handler, t),
    }
}

fn condition(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    // let condition_node = TokenNode::new(NodeType::Condition());
    // token_handler.next_token();
    println!("opening condition token: {:?}", token_handler.get_token());
    match token_handler.get_token() {
        Token::OParen => {
            // evaluate condition
            token_handler.next_token();
            let condition = condition_expr(token_handler);
            //token_handler.next_token();
            match token_handler.get_token() {
                Token::CParen => condition,
                _ => {
                    println!("post condition {:?}", token_handler.get_token());
                    Err(token_handler.new_err(ET::ExpectedCParen))
                }
            }
        }
        _ => Err(token_handler.new_err(ET::ExpectedOParen)),
    }
}

fn condition_expr(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    println!("CONDITION expr");
    let mut left = condition_term(token_handler)?;
    if token_handler.peek(1) != Token::AndCmp && token_handler.peek(1) != Token::OrCmp {
        return Ok(left);
    }
    token_handler.next_token();
    let mut curr = token_handler.get_token();
    while *curr == Token::AndCmp || *curr == Token::OrCmp {
        let cmp = curr.clone();
        let right = condition_term(token_handler)?;
        let cmp_tok = TokenNode::new(NodeType::from_token(cmp).unwrap(), Some(vec![left, right]));

        left = cmp_tok;
        // token_handler.next_token();
        curr = token_handler.get_token();
        println!("{:?}", curr);
    }
    Ok(left)
}

fn condition_term(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    println!("Condition term");
    let mut left = condition_factor(token_handler)?;
    if token_handler.peek(1) != Token::NeqCmp && token_handler.peek(1) != Token::EqCmp {
        return Ok(left);
    }
    token_handler.next_token();
    let mut curr = token_handler.get_token();
    while *curr == Token::NeqCmp || *curr == Token::EqCmp {
        let cmp: Token = curr.clone();
        println!("condition cmp: {:?}", cmp);
        token_handler.next_token();
        let right = condition_factor(token_handler)?;
        let cmp_tok = TokenNode::new(NodeType::from_token(cmp).unwrap(), Some(vec![left, right]));

        left = cmp_tok;
        token_handler.next_token();
        curr = &token_handler.get_token();
        println!("{:?}", curr);
    }
    Ok(left)
}

fn condition_factor(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    println!("Condition factor: {:?}", token_handler.get_token());
    match token_handler.get_token() {
        Token::NumLiteral(num) => Ok(TokenNode::new(NodeType::NumLiteral(*num), None)),
        Token::Id(name) => Ok(TokenNode::new(NodeType::Id(name.clone()), None)),
        Token::OParen => {
            token_handler.next_token();
            let node = expression(token_handler)?;
            if *token_handler.get_token() == Token::CParen {
                Ok(node)
            } else {
                Err(token_handler.new_err(ET::ExpectedCParen))
            }
        }
        _ => Err(token_handler.new_err(ET::ExpectedCondition)),
    }
}

fn asm_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    token_handler.next_token();
    if *token_handler.get_token() != Token::OParen {
        return Err(token_handler.new_err(ET::ExpectedOParen));
    }
    token_handler.next_token();
    match token_handler.get_token().clone() {
        Token::StrLiteral(str) => {
            token_handler.next_token();
            if *token_handler.get_token() != Token::CParen {
                return Err(token_handler.new_err(ET::ExpectedCParen));
            }
            token_handler.next_token();
            if *token_handler.get_token() != Token::Semi {
                return Err(token_handler.new_err(ET::ExpectedSemi));
            }
            return Ok(TokenNode::new(NodeType::Asm(str.to_string()), None));
        }
        _ => return Err(token_handler.new_err(ET::ExpectedStrLiteral)),
    }
}

/// TODO: Make this function check for semi-colonons
fn for_statement(token_handler: &mut TokenHandler) -> Result<TokenNode, RhErr> {
    let mut for_node = TokenNode::new(NodeType::If, Some(vec![]));
    let t = match token_handler.get_token() {
        Token::Type(t) => t,
        _ => return Err(token_handler.new_err(ET::ExpectedType)),
    };
    let declare_node = declaration(token_handler, t.clone())?;
    for_node
        .children
        .as_mut()
        .expect("vec to be some")
        .push(declare_node);
    println!("token: {:?}, should be ;", token_handler.get_token());
    token_handler.next_token(); // might make semi handled by the called functions instead
    let condition_node = condition(token_handler)?;
    for_node
        .children
        .as_mut()
        .expect("vec to be some")
        .push(condition_node);
    token_handler.next_token();
    let statement_node = statement(token_handler, ScopeType::For)?;
    for_node
        .children
        .as_mut()
        .expect("vec to be some")
        .push(statement_node);
    token_handler.next_token();
    Ok(for_node)
}
