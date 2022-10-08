use super::EditorContext;

pub trait EditorCommand {
    fn undo(&self, context: &mut EditorContext) -> Box<dyn EditorCommand>;
}
