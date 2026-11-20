use crate::{
    ast::*,
    fail::{Errors, Fail},
    lex::{self, Loc, Span, Tok, Token},
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
                report(Tok::Fun.as_str(), tok, next.loc, &mut ctx);
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

fn report(expected: &str, tok: Tok, loc: Loc, ctx: &mut Context) -> Fail {
    ctx.errors.parse(&ctx.stream.source, loc, expected, tok)
}

fn fun(ctx: &mut Context) -> Result<Fun, Fail> {
    let name = expect(Tok::Name, ctx)?;
    expect(Tok::Parl, ctx)?;
    expect(Tok::Parr, ctx)?;
    Ok(Fun {
        name,
        ret_typ: typ(ctx)?,
        body: block(ctx)?,
    })
}

fn typ(ctx: &mut Context) -> Result<Typ, Fail> {
    let next = ctx.stream.next();
    match next.tok {
        Tok::Name => Ok(Typ::from_span(next.span, next.loc, &ctx.stream.source.code)),
        _ => unreachable!("parse typ with first tok {}", next.tok.as_str()),
    }
}

fn block(ctx: &mut Context) -> Result<Block, Fail> {
    expect(Tok::Curl, ctx)?;
    let mut exprs = Vec::new();
    loop {
        let (expr, next) = expr(ctx)?;
        match next.tok {
            Tok::Curr => break Ok(Block { exprs, out: expr }),
            Tok::Semi => exprs.push(expr),
            _ => break Err(report(Tok::Semi.as_str(), next.tok, next.loc, ctx)),
        }
    }
}

fn expr(ctx: &mut Context) -> Result<(Expr, Token), Fail> {
    let next = ctx.stream.next();
    expr_with(next, ctx)
}

fn expr_named(next: Token, ctx: &mut Context) -> Result<Expr, Fail> {
    let name = next.span;
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
            _ => return Err(report(Tok::Comma.as_str(), next.tok, next.loc, ctx)),
        }
    }
    Ok(Call { name, args }.into())
}

fn expr_with(next: Token, ctx: &mut Context) -> Result<(Expr, Token), Fail> {
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

fn expr_atom_with(next: Token, ctx: &mut Context) -> Result<Expr, Fail> {
    match next.tok {
        Tok::Str => Ok(Expr::Str(next.span)),
        Tok::Int => Ok(Int {
            val: ctx.stream.source.code[next.span].parse().unwrap(),
            loc: next.loc,
        }
        .into()),
        Tok::Name => expr_named(next, ctx),
        _ => Err(report("<expr>", next.tok, next.loc, ctx)),
    }
}

fn expect(expected: Tok, ctx: &mut Context) -> Result<Span, Fail> {
    let next = ctx.stream.next();
    expect_with(next, expected, ctx)
}

fn expect_with(
    next: Token,
    expected: Tok,
    ctx: &mut Context,
) -> Result<std::range::Range<usize>, Fail> {
    if next.tok == expected {
        Ok(next.span)
    } else {
        Err(report(expected.as_str(), next.tok, next.loc, ctx))
    }
}

struct Context {
    stream: lex::Stream,
    errors: Errors,
}

impl Context {
    fn new(source: Source) -> Self {
        Self {
            errors: Errors::new(),
            stream: lex::Stream::new(source),
        }
    }
}
