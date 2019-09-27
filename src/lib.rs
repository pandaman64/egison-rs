use refl::Id;
use frunk::{Hlist, HCons, HNil};

use std::collections::VecDeque;

mod matching;

pub trait Append<Other> {
    type Output;

    fn append(self, other: Other) -> Self::Output;
}

type AppendOutput<Lhs, Rhs> = <Lhs as Append<Rhs>>::Output;

impl<Other> Append<Other> for HNil {
    type Output = Other;

    fn append(self, other: Other) -> Self::Output {
        other
    }
}

impl<Other, Head, Tail> Append<Other> for HCons<Head, Tail>
where
    Tail: Append<Other>,
{
    type Output = HCons<Head, AppendOutput<Tail, Other>>;

    fn append(self, other: Other) -> Self::Output {
        let (head, tail) = self.pop();
        HCons {
            head,
            tail: tail.append(other),
        }
    }
}

pub enum Placeholder {}

// Pattern type
// A: target
// M: matcher
// Ctx: intermediate pattern-matching result
// Vs: the list of types bound to the pattern variables in the pattern
pub enum Pattern<A, M, Ctx, Vs> {
    Wildcard(Id<Vs, HNil>), // Wildcard :: Pattern a m ctx '[]
    PatVar(String, Id<Vs, Hlist![A]>),
    AndPat(Box<dyn AndPat<A, M, Ctx, Vs>>),
    OrPat(Box<Self>, Box<Self>),
    NotPat(Box<Self>, Id<Vs, HNil>),
    PredicatePat(Box<dyn Fn(Ctx, A) -> bool>, Id<Vs, HNil>),
    Pattern(Box<dyn Fn(Ctx, M, A) -> Vec<Placeholder>>),
}

pub trait AndPat<A, M, Ctx, Vs> {
}

pub struct AndPatImpl<A, M, Ctx, Vs1, Vs2>
where
    Ctx: Append<Vs1>,
{
    pub lhs: Pattern<A, M, Ctx, Vs1>,
    pub rhs: Pattern<A, M, AppendOutput<Ctx, Vs1>, Vs2>,
}

impl<A, M, Ctx, Vs1, Vs2> AndPat<A, M, Ctx, AppendOutput<Vs1, Vs2>> for AndPatImpl<A, M, Ctx, Vs1, Vs2>
where
    Ctx: Append<Vs1>,
    Vs1: Append<Vs2>,
{
}

pub trait MatchClause<A, M, B> {
}

pub struct MatchClauseImpl<A, M, B, Vs> {
    pub pattern: Pattern<A, M, HNil, Vs>,
    pub processor: Box<dyn Fn(Vs) -> B>,
}

impl<A, M, B, Vs> MatchClause<A, M, B> for MatchClauseImpl<A, M, B, Vs> {
}

pub trait MState<Vs> {
    fn decompose_if_nil(&self) -> Option<Vs>;
    fn process(&self) -> VecDeque<Box<dyn MState<Vs>>>;
}

pub struct MStateImpl<Xs, Ys> {
    pub rs: Xs,
    pub list: MList<Xs, Ys>,
}

impl<Xs, Ys> MState<AppendOutput<Xs, Ys>> for MStateImpl<Xs, Ys>
where
    Xs: Append<Ys> + Clone,
{
    fn decompose_if_nil(&self) -> Option<AppendOutput<Xs, Ys>> {
        use MList::*;

        match self.list {
            MNil(refl) => {
                Some(self.rs.clone().append(refl.sym().cast(HNil)))
            }
            _ => None,
        }
    }

    fn process(&self) -> VecDeque<Box<dyn MState<AppendOutput<Xs, Ys>>>> {
        use MList::*;

        match &self.list {
            MNil(_) => unreachable!(),
            MCons(mcons) => mcons.process(self.rs.clone()),
            MJoin(mjoin) => unimplemented!(), //mjoin.process(),
        }
    }
}

pub trait MAtom<Ctx, Vs> {
}

pub struct MAtomImpl<A, M, Ctx, Vs> {
    pub pattern: Pattern<A, M, Ctx, Vs>,
    pub matcher: M,
    pub target: A,
}

impl<A, M, Ctx, Vs> MAtom<Ctx, Vs> for MAtomImpl<A, M, Ctx, Vs> {
}

pub enum MList<Ctx, Vs> {
    MNil(Id<Vs, HNil>),
    MCons(Box<dyn MCons<Ctx, Vs>>),
    MJoin(Box<dyn MJoin<Ctx, Vs>>),
}

pub trait MCons<Ctx, Vs>
where
    Ctx: Append<Vs>,
{
    fn process(&self, rs: Ctx) -> VecDeque<Box<dyn MState<AppendOutput<Ctx, Vs>>>>;
}

pub struct MConsImpl<Ctx, Xs, Ys>
where
    Ctx: Append<Xs>,
{
    head: Box<dyn MAtom<Ctx, Xs>>,
    tail: MList<AppendOutput<Ctx, Xs>, Ys>
}

impl<Ctx, Xs, Ys> MCons<Ctx, AppendOutput<Xs, Ys>> for MConsImpl<Ctx, Xs, Ys>
where
    Ctx: Append<Xs> + Append<AppendOutput<Xs, Ys>>,
    Xs: Append<Ys>,
{
    fn process(&self, rs: Ctx) -> VecDeque<Box<dyn MState<AppendOutput<Ctx, AppendOutput<Xs, Ys>>>>> {
        unimplemented!()
    }
}

pub trait MJoin<Ctx, Vs> 
where
    Ctx: Append<Vs>,
{
    fn process(&self, rs: Ctx) -> VecDeque<Box<dyn MState<AppendOutput<Ctx, Vs>>>>;
}

pub struct MJoinImpl<Ctx, Xs, Ys>
where
    Ctx: Append<Xs>,
{
    lhs: MList<Ctx, Xs>,
    rhs: MList<AppendOutput<Ctx, Xs>, Ys>,
}

impl<Ctx, Xs, Ys> MJoin<Ctx, AppendOutput<Xs, Ys>> for MJoinImpl<Ctx, Xs, Ys>
where
    Ctx: Append<Xs> + Append<AppendOutput<Xs, Ys>> + Clone,
    Xs: Append<Ys> + Clone,
    MList<AppendOutput<Ctx, Xs>, Ys>: Clone,
{
    fn process(&self, rs: Ctx) -> VecDeque<Box<dyn MState<AppendOutput<Ctx, AppendOutput<Xs, Ys>>>>> {
        use MList::*;

        match &self.lhs {
            MNil(refl) => {
                let ms: MStateImpl<Ctx, AppendOutput<Xs, Ys>> = MStateImpl {
                    rs,
                    list: self.rhs.clone(),
                };
                ms.process()
            }
            lhs => unimplemented!()
        }
    }
}
