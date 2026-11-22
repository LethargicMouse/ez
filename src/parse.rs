use crate::{
    ast::*,
    fail::{Errors, Fail},
    lex::{self, Pos, Tok, Token},
};

pub fn run(source: Source) -> Result<Module, Fail> {
    module(Context::new(source))
}

fn module(mut ctx: Context) -> Result<Module, Fail> {
    let mut funs = Vec::new();
    loop {
        let next = ctx.stream.next();
        match next.tok {
            Tok::Eof => break,
            Tok::Fun => match fun(&mut ctx) {
                Ok(fun) => funs.push(fun),
                Err(_) => break,
            },
            tok => {
                report(Tok::Fun.as_str(), tok, next.pos, &mut ctx);
                break;
            }
        }
    }
    if ctx.errors.present() {
        Err(Fail)
    } else {
        Ok(Module {
            source: ctx.stream.source,
            funs,
        })
    }
}

fn report(expected: &str, tok: Tok, pos: Pos, ctx: &mut Context) -> Fail {
    ctx.errors.parse(&ctx.stream.source, pos, expected, tok)
}

fn fun<'a>(ctx: &mut Context<'a>) -> Result<Fun<'a>, Fail> {
    let name = expect(Tok::Name, ctx)?;
    expect(Tok::Parl, ctx)?;
    expect(Tok::Parr, ctx)?;
    Ok(Fun {
        name,
        ret_typ: typ(ctx)?,
        body: block(ctx)?,
    })
}

fn typ<'a>(ctx: &mut Context<'a>) -> Result<Typ<'a>, Fail> {
    let next = ctx.stream.next();
    match next.tok {
        Tok::Name => Ok(Typ::from_name(next.body, next.pos)),
        _ => unreachable!("parse typ with first tok {}", next.tok.as_str()),
    }
}

fn block<'a>(ctx: &mut Context<'a>) -> Result<Block<'a>, Fail> {
    expect(Tok::Curl, ctx)?;
    let mut exprs = Vec::new();
    loop {
        let (expr, next) = expr(ctx)?;
        match next.tok {
            Tok::Curr => break Ok(Block { exprs, out: expr }),
            Tok::Semi => exprs.push(expr),
            _ => break Err(report(Tok::Semi.as_str(), next.tok, next.pos, ctx)),
        }
    }
}

fn expr<'a>(ctx: &mut Context<'a>) -> Result<(Expr<'a>, Token<'a>), Fail> {
    let next = ctx.stream.next();
    expr_with(next, ctx)
}

fn expr_named<'a>(next: Token<'a>, ctx: &mut Context<'a>) -> Result<Expr<'a>, Fail> {
    let name = next.body;
    let pos = next.pos;
    expect(Tok::Parl, ctx)?;
    let mut args = Vec::new();
    loop {
        let next = ctx.stream.next();
        if next.tok == Tok::Parr {
            break;
        }
        let (expr, next) = expr_with(next, ctx)?;
        args.push(expr);
        match next.tok {
            Tok::Comma => {}
            Tok::Parr => break,
            _ => return Err(report(Tok::Comma.as_str(), next.tok, next.pos, ctx)),
        }
    }
    Ok(Call { name, args, pos }.into())
}

fn expr_with<'a>(next: Token<'a>, ctx: &mut Context<'a>) -> Result<(Expr<'a>, Token<'a>), Fail> {
    let expr = expr_atom_with(next, ctx)?;
    loop {
        let next = ctx.stream.next();
        match next.tok {
            Tok::Dot => {
                let name = expect(Tok::Name, ctx)?;
            }
            _ => break Ok((expr, next)),
        }
    }
}

fn expr_atom_with<'a>(next: Token<'a>, ctx: &mut Context<'a>) -> Result<Expr<'a>, Fail> {
    match next.tok {
        Tok::Str => Ok(Expr::Str(next.body)),
        Tok::Int => Ok(Int {
            val: next.body.parse().unwrap(),
            pos: next.pos,
        }
        .into()),
        Tok::Name => expr_named(next, ctx),
        _ => Err(report("<expr>", next.tok, next.pos, ctx)),
    }
}

fn expect<'a>(expected: Tok, ctx: &mut Context<'a>) -> Result<&'a str, Fail> {
    let next = ctx.stream.next();
    expect_with(next, expected, ctx)
}

fn expect_with<'a>(next: Token<'a>, expected: Tok, ctx: &mut Context<'a>) -> Result<&'a str, Fail> {
    if next.tok == expected {
        Ok(next.body)
    } else {
        Err(report(expected.as_str(), next.tok, next.pos, ctx))
    }
}

struct Context<'a> {
    stream: lex::Stream<'a>,
    errors: Errors,
}

impl<'a> Context<'a> {
    fn new(source: Source<'a>) -> Self {
        Self {
            errors: Errors::new(),
            stream: lex::Stream::new(source),
        }
    }
}
