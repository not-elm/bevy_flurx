
use crate::prelude::TaskAction;

macro_rules! sequence {
    ($action: expr) => {$action};
    ($action1: expr, $action2: expr $(,$action: expr)*$(,)?) => {
        
    };
}

pub trait Then<In, Out> {
    fn then<O>(self, next: impl TaskAction<In=Out, Out=O>) -> impl TaskAction<In=In, Out=O>;
}

pub struct ThenAction<First, Second> {
    first: First,
    second: Second,
}

// impl<
//     First, I1, O1,
//     Second, O2
// > TaskAction for ThenAction<First, Second>
//     where
//         First: TaskAction<In=I1, Out=O1>,
//         Second: TaskAction<In=O1, Out=O2>
// {
//     type In = I1;
//
//     type Out = O2;
//
//     fn to_runner(self, output: TaskOutput<Self::Out>) -> impl RunTask {
//         let o = Rc::new()
//     }
// }





