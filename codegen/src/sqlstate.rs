use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Debug)]
enum Kind {
    Error,
    Warning,
    Success,
}

struct Error {
    code: String,
    kind: Kind,
    name: String,
    message: Option<String>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{doc}
pub const {name}: State = State {{
    code: "{code}",
    name: "{name}",
    kind: Kind::{kind:?},
    message: {message},
}};
"#,
            code = self.code,
            name = self.name,
            kind = self.kind,
            message = self
                .message
                .as_ref()
                .map(|x| format!("Some(\"{x}\")"))
                .unwrap_or_else(|| "None".to_string()),
            doc = self
                .message
                .as_ref()
                .map(|x| format!("/// {x}"))
                .unwrap_or_default(),
        )
    }
}

const ERRCODES_TXT: &str = include_str!("errcodes.txt");

pub fn build(filename: &str) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(filename)?);

    let errors = parse_errors();

    make_header(&mut file)?;
    make_consts(&errors, &mut file)?;
    make_type(&errors, &mut file)
}

fn parse_errors() -> BTreeMap<String, Error> {
    let mut errors = BTreeMap::new();

    for line in ERRCODES_TXT.lines() {
        if line.starts_with('#') || line.starts_with("Section") || line.trim().is_empty() {
            continue;
        }

        let mut it = line.split_whitespace();
        let code = it.next().unwrap().to_string();
        let kind = match it.next().unwrap() {
            "E" => Kind::Error,
            "W" => Kind::Warning,
            "S" => Kind::Success,
            _ => unreachable!(),
        };
        let name = it.next().unwrap().replace("ERRCODE_", "");
        let message = it.next().map(|x| x.replace('_', " "));

        let error = Error {
            code: code.clone(),
            kind,
            name,
            message,
        };

        errors.insert(code, error);
    }

    errors
}

fn make_header(file: &mut BufWriter<File>) -> std::io::Result<()> {
    writeln!(file, "// Autogenerated file - DO NOT EDIT")
}

fn make_type(errors: &BTreeMap<String, Error>, file: &mut BufWriter<File>) -> std::io::Result<()> {
    let mut from_code = Vec::new();

    for (id, error) in errors {
        from_code.push(format!("            \"{id}\" => {},", error.name));
    }

    write!(
        file,
        "
impl State {{
    /// Creates a `State` from its error code.
    pub fn from_code(s: &str) -> State {{
        match s {{
{}
            _ => unreachable!(),
        }}
    }}
}}
",
        from_code.join("\n")
    )
}

fn make_consts(
    errors: &BTreeMap<String, Error>,
    file: &mut BufWriter<File>,
) -> std::io::Result<()> {
    for error in errors.values() {
        write!(file, "{error}")?;
    }

    Ok(())
}
