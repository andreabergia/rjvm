// #[derive(Debug)]
// pub struct Stack<'a> {
//     vm: &'a mut Vm<'a>,
//     frames: Vec<Rc<RefCell<CallFrame<'a>>>>,
// }
//
// impl<'a> Stack<'a> {
//     pub fn new(vm: &'a mut Vm<'a>) -> Stack<'a> {
//         Stack {
//             vm,
//             frames: Vec::new(),
//         }
//     }
//
//     pub fn add_frame(
//         &mut self,
//         class_and_method: &'a ClassAndMethod,
//         receiver: Option<&'a Value>,
//         args: Vec<&'a Value>,
//     ) -> Rc<RefCell<CallFrame<'a>>> {
//         // TODO: verify local size with static method data
//         let locals = receiver.into_iter().chain(args.into_iter()).collect();
//         let new_frame = CallFrame::new(self.vm, class_and_method, locals);
//         let new_frame = Rc::new(RefCell::new(new_frame));
//         self.frames.push(Rc::clone(&new_frame));
//         new_frame
//     }
// }
