use tuples_op::invariant;

trait PartTrait<'s> {
    type Return;
    fn method(&'s mut self, ctx: &str) -> Self::Return;
}

// invariant! {
//     trait TupleTrait {
//         fn part_mut_ctx_mut<E: Mutate>(part: &mut E, ctx: &mut String) {
//             if INDEX != 0 {
//                 ctx.push_str(", ");
//                 part.my_behavior(ctx);
//             }else {
//                 part.my_behavior(ctx);
//             }
//         }
//         fn my_behavior(&mut self) -> String {
//             let mut output = String::new();
//             self.tuple_mut_ctx_mut(&mut output);
//             output
//         }
//     }
// }
