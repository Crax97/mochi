use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    fs::File,
    io::{BufRead, BufReader, Cursor, Error, Read},
};

use naga::{
    front::wgsl::{parse_str, ParseError},
    valid::{Capabilities, ValidationFlags, Validator},
    Module,
};
use wgpu::ShaderModuleDescriptor;

enum PreprocessorCommand {
    Nothing(String),
    IncludeFile(String),
    IncludeDefinition(String),
}

enum PreprocessError {
    UnrecognizedCommand(String),
    EmptyCommand,
    EmptyInclude,
    FileNotAccessible(String, Error),
    DefinitionNotFound(String),
}
impl Debug for PreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreprocessError::UnrecognizedCommand(cmd) => {
                f.write_fmt(format_args!("Unrecognized preprocessor command: {cmd}"))
            }
            PreprocessError::EmptyCommand => f.write_str("Empty preprocessor command!"),
            PreprocessError::EmptyInclude => f.write_str("Empty include statement!"),
            PreprocessError::FileNotAccessible(file_path, why) => f.write_fmt(format_args!(
                "File not accessible: {file_path}, because {}",
                why.to_string()
            )),
            PreprocessError::DefinitionNotFound(definition_name) => {
                f.write_fmt(format_args!("Non-existent definition: {definition_name}"))
            }
        }
    }
}
impl Display for PreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(&self, f)
    }
}
impl std::error::Error for PreprocessError {}

pub struct ShaderCompiler {
    definitions: HashMap<String, String>,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    // validates source and, if valid, stores it as a definition to be later used for subsequent
    // compile calls
    pub fn define<T: Into<String>>(&mut self, name: T, source: T) -> anyhow::Result<()> {
        let source = source.into();
        let compiled_module = self.compile(&source)?;
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        let _info = validator.validate(&compiled_module)?;
        // print info;
        self.definitions.insert(name.into(), source);
        Ok(())
    }

    // Preprocesses and compiles the given source
    pub fn compile(&self, source: &str) -> Result<Module, ParseError> {
        let parsed_source = self.preprocess_source(source);
        parse_str(&parsed_source)
    }

    pub fn compile_into_shader_description<'a>(
        &'a self,
        name: &'a str,
        source: &str,
    ) -> Result<ShaderModuleDescriptor, ParseError> {
        let module = self.compile(source);
        module.and_then(|module| {
            Ok(ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Naga(module),
            })
        })
    }

    fn preprocess_source(&self, source: &str) -> String {
        let cursor = Cursor::new(source);
        let lines = cursor.lines();
        let lines: Vec<_> = lines
            .map(|line| self.parse_command(line.unwrap()))
            .map(|line| {
                if let Err(e) = &line {
                    panic!("While preprocessing, {}", e)
                } else {
                    line.unwrap()
                }
            })
            .map(|command| self.execute_command(command))
            .map(|line| {
                if let Err(e) = &line {
                    panic!("While preprocessing, {}", e)
                } else {
                    line.unwrap()
                }
            })
            .collect();
        lines.join("\n")
    }

    fn parse_command(&self, line: String) -> Result<PreprocessorCommand, PreprocessError> {
        #[derive(Eq, PartialEq)]
        enum ParseState {
            Whitespaces,
            FirstSlash,
            SecondSlash,
            ParsingCommand,
        }
        let mut command = String::new();
        let mut stage = ParseState::Whitespaces;
        for ch in line.chars() {
            match stage {
                ParseState::Whitespaces => {
                    if ch == ' ' {
                        continue;
                    } else if ch == '/' {
                        stage = ParseState::FirstSlash
                    } else {
                        return Ok(PreprocessorCommand::Nothing(line));
                    }
                }
                ParseState::FirstSlash => {
                    if ch == '/' {
                        stage = ParseState::SecondSlash
                    } else {
                        return Ok(PreprocessorCommand::Nothing(line));
                    }
                }
                ParseState::SecondSlash => {
                    if ch == '@' {
                        stage = ParseState::ParsingCommand
                    } else {
                        return Ok(PreprocessorCommand::Nothing(line));
                    }
                }
                ParseState::ParsingCommand => {
                    command.push(ch);
                }
            }
        }

        if command.is_empty() {
            if stage == ParseState::ParsingCommand {
                return Err(PreprocessError::EmptyCommand);
            } else {
                // Just an empty line
                return Ok(PreprocessorCommand::Nothing(line));
            }
        }

        let mut command_and_args = command.split(" ");
        let command = command_and_args.next().unwrap();
        match command {
            "include" => {
                let arg = match command_and_args.next() {
                    Some(arg) => arg,
                    None => return Err(PreprocessError::EmptyInclude),
                };
                if arg.is_empty() {
                    return Err(PreprocessError::EmptyInclude);
                }
                if arg.chars().into_iter().next().unwrap() == ':' {
                    return Ok(PreprocessorCommand::IncludeDefinition(arg[1..].to_owned()));
                } else {
                    return Ok(PreprocessorCommand::IncludeFile(arg.to_owned()));
                }
            }
            _ => Err(PreprocessError::UnrecognizedCommand(command.to_owned())),
        }
    }

    fn execute_command(&self, command: PreprocessorCommand) -> Result<String, PreprocessError> {
        match command {
            PreprocessorCommand::Nothing(s) => Ok(s),
            PreprocessorCommand::IncludeFile(file_path) => self.include_file_path(file_path),
            PreprocessorCommand::IncludeDefinition(def) => self.include_definition(def),
        }
    }

    fn include_file_path(&self, file_path: String) -> Result<String, PreprocessError> {
        let file = File::open(&file_path);
        if let Err(e) = file {
            return Err(PreprocessError::FileNotAccessible(file_path, e));
        }
        let file = file.unwrap();
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        if let Err(e) = reader.read_to_string(&mut content) {
            return Err(PreprocessError::FileNotAccessible(file_path, e));
        }
        Ok(content)
    }

    fn include_definition(&self, def: String) -> Result<String, PreprocessError> {
        if let Some(definition) = self.definitions.get(&def) {
            Ok(definition.clone())
        } else {
            Err(PreprocessError::DefinitionNotFound(def))
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::{
        fs::{remove_file, File},
        io::{LineWriter, Write},
    };

    use super::*;

    #[test]
    pub fn compile_empty() {
        let compiler = ShaderCompiler::new();
        assert!(compiler.compile("").is_ok())
    }

    #[test]
    pub fn compile_simple() {
        let compiler = ShaderCompiler::new();
        let module = compiler.compile(
            "        
        @fragment
        fn fragment() -> @location(0) vec4<f32> {
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }
        ",
        );
        assert!(module.is_ok());
        assert!(module
            .unwrap()
            .entry_points
            .iter()
            .any(|e| e.name == "fragment"));
    }

    #[test]
    pub fn compile_include_string() {
        write_test_include("test.wgsl");
        let compiler = ShaderCompiler::new();
        let module = compiler.compile(
            "        
            //@include test.wgsl
        ",
        );
        assert!(module.is_ok());
        assert!(module
            .unwrap()
            .entry_points
            .iter()
            .any(|e| e.name == "fragment"));

        let _ = remove_file("test.wgsl");
    }

    fn write_test_include(arg: &str) {
        let output = File::create(arg).unwrap();
        let mut writer = LineWriter::new(output);
        writer
            .write_all(
                b"@fragment
        fn fragment() -> @location(0) vec4<f32> {
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }",
            )
            .unwrap();
    }
    #[test]
    pub fn compile_include_definition() {
        let mut compiler = ShaderCompiler::new();
        assert!(compiler
            .define(
                "test",
                "@fragment
        fn fragment() -> @location(0) vec4<f32> {
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }"
            )
            .is_ok());
        let module = compiler.compile(
            "        
            //@include :test
        ",
        );
        assert!(module.is_ok());
        assert!(module
            .unwrap()
            .entry_points
            .iter()
            .any(|e| e.name == "fragment"));
    }
}
