use crate::{
    ast::*,
    fail::{Errors, Fail},
    lex::{self, Loc, Span, Tok},
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
        Tok::Name => {
            let next = ctx.stream.next();
            Ok(Typ::from_span(next.span, next.loc, &ctx.stream.source.code))
        }
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
    let next = ctx.stream.next();
    match next.tok {
        Tok::Int => Ok(int_expr(next.span, next.loc, ctx)?.into()),
        _ => Err(report("<expr>", next.tok, next.loc, ctx)),
    }
}

fn int_expr(span: Span, loc: Loc, ctx: &mut Context) -> Result<Int, Fail> {
    Ok(Int {
        val: ctx.stream.source.code[span].parse().unwrap(),
        loc,
    })
}

fn expect(expected: Tok, ctx: &mut Context) -> Result<Span, Fail> {
    let next = ctx.stream.next();
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
