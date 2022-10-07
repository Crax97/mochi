use super::EditorContext;

pub trait EditorCommand {
    fn execute(&self, editor_context: &mut EditorContext);
    fn undo(&self) -> Box<dyn EditorCommand>;
}
