use crate::{
    ast::*,
    fail::{Errors, Fail},
    lex::{self, Span, Tok},
};

pub fn run(source: Source) -> Result<Module, Fail> {
    module(Context::new(source))
}

fn module(mut ctx: Context) -> Result<Module, Fail> {
    let mut funs = Vec::new();
    loop {
        match ctx.stream.peek().tok {
            Tok::Eof => break,
            Tok::Fun => match fun(&mut ctx) {
                Ok(fun) => funs.push(fun),
                Err(_) => break,
            },
            tok => {
                report(Tok::Fun.as_str(), tok, &mut ctx);
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

fn report(expected: &str, tok: Tok, ctx: &mut Context) -> Fail {
    let token = ctx.stream.peek();
    ctx.errors
        .parse(&ctx.stream.source, token.loc, expected, tok)
}

fn fun(ctx: &mut Context) -> Result<Fun, Fail> {
    ctx.stream.consume(Tok::Fun); // Tok::Fun
    let name = expect(Tok::Name, ctx)?;
    expect(Tok::Parl, ctx)?;
    expect(Tok::Parr, ctx)?;
    Ok(Fun {
        name,
        ret_typ: if let Some(tok) = peeks_typ(ctx) {
            Some(typ(tok, ctx)?)
        } else {
            None
        },
        body: block(ctx)?,
    })
}

fn typ(fst: Tok, ctx: &mut Context) -> Result<Typ, Fail> {
    match fst {
        Tok::Name => Ok(Typ::from_span(
            ctx.stream.next().span,
            &ctx.stream.source.code,
        )),
        _ => unreachable!("parse typ with first tok {}", fst.as_str()),
    }
}

fn peeks_typ(ctx: &mut Context) -> Option<Tok> {
    let tok = ctx.stream.peek().tok;
    match tok {
        Tok::Name => Some(tok),
        _ => None,
    }
}

fn block(ctx: &mut Context) -> Result<Block, Fail> {
    expect(Tok::Curl, ctx)?;
    let out = expr(ctx)?;
    expect(Tok::Curr, ctx)?;
    Ok(Block { out })
}

fn expr(ctx: &mut Context) -> Result<Expr, Fail> {
    match ctx.stream.peek().tok {
        Tok::Int => Ok(int_expr(ctx)?.into()),
        tok => Err(report("<expr>", tok, ctx)),
    }
}

fn int_expr(ctx: &mut Context) -> Result<Int, Fail> {
    let token = ctx.stream.next();
    assert_eq!(Tok::Int, token.tok);
    Ok(Int {
        val: ctx.stream.source.code[token.span].parse().unwrap(),
        loc: token.loc,
    })
}

fn expect(expected: Tok, ctx: &mut Context) -> Result<Span, Fail> {
    let tok = ctx.stream.peek().tok;
    if tok == expected {
        Ok(ctx.stream.next().span)
    } else {
        Err(report(expected.as_str(), tok, ctx))
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
