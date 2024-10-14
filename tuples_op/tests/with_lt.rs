use std::fmt::Display;

use tuples_op::invariant;

trait WithLifetimes<'q, G: Display> {
    fn my_behavior(&mut self, ctx: &mut String);
}

// invariant! {
//     trait MutateExt<'q, G: Display> {
//         fn each<E: WithLifetimes<'q, G>>(part: &mut E, ctx: &mut String) {
//             if INDEX != 0 {
//                 ctx.push_str(", ");
//                 part.my_behavior(ctx);
//             }else {
//                 part.my_behavior(ctx);
//             }
//         }
//         fn my_behavior(&mut self) -> String {
//             let mut output = String::new();
//             self.mut_each_mut_ctx(&mut output);
//             output
//         }
//     }
// }
