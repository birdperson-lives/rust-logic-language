#![allow(dead_code)]

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::result;


// pub trait Error {
//     const ERROR_NAME: &'static str;

//     fn location_string(&self) -> String;

//     fn message(&self) -> String;

//     fn to_console<W: Write>(&self, dest: &mut W) {
//         dest.write(format!("At {}:\n{} : {}\n", self.location_string(),
//             <Self as Error>::ERROR_NAME, self.message()).as_bytes());
//     }
// }


// pub struct FileReadFailure {
//     filename: String,
//     line: usize,
//     ekind: Option<io::ErrorKind>,
// }

// impl FileReadFailure {
//     pub fn new(filename: &str, line: usize, ekind: Option<io::ErrorKind>) -> FileReadFailure {
//         FileReadFailure {
//             filename: String::from(filename),
//             line: line,
//             ekind: ekind.clone(),
//         }
//     }
// }

// impl Error for FileReadFailure {
//     const ERROR_NAME: &'static str = "FileReadFailure";

//     fn location_string(&self) -> String {
//         format!("{}:{}", self.filename.clone(), self.line)
//     }

//     fn message(&self) -> String {
//         if let Some(ioerr) = self.ekind {
//             format!("could not read source file: {:?}", ioerr)
//         } else {
//             String::from("could not open source file")
//         }
//     }
// }


#[derive(Debug)]
pub enum ErrorKind {
    FileOpenFailure {
        filename: String,
        rust_err: io::ErrorKind,
    },
    FileReadFailure {
        filename: String,
        line: usize,
        rust_err: io::ErrorKind,
    },
    UnexpectedToken {
        found: String,
        expected: Vec<String>,
    },
    NoBinding {
        name: String,
    },
    BindingExists {
        name: String,
    },
    ITypeMismatch {
        found: String,
        expected: String,
    },
    MTypeMismatch {
        found: String,
        expected: String,
    },
    UnboundImplication,
    UnboundTheorem,
}

use self::ErrorKind::*;


pub struct Error {
    error: ErrorKind,
    location: FileLocation,
}

impl Error {
    pub fn new(error: ErrorKind, location: &FileLocation) -> Error {
        Error {
            error: error,
            location: location.clone(),
        }
    }

    fn location_string(&self) -> String {
        self.location.full_string()
    }

    fn err_type(&self) -> &'static str {
        match self.error {
            FileOpenFailure{..} => "FileOpenFailure"   ,
            FileReadFailure{..} => "FileReadFailure"   ,
            UnexpectedToken{..} => "UnexpectedToken"   ,
            NoBinding{..}       => "NoBinding"         ,
            BindingExists{..}   => "BindingExists"     ,
            ITypeMismatch{..}   => "ITypeMismatch"     ,
            MTypeMismatch{..}   => "MTypeMismatch"     ,
            UnboundImplication  => "UnboundImplication",
            UnboundTheorem      => "UnboundTheorem"    ,
        }
    }

    fn message(&self) -> String {
        match self.error {
            FileOpenFailure {
                ref filename,
                rust_err,
            } => format!("could not open source file \"{}\" because of error `{:?}`",
                filename, rust_err),
            FileReadFailure {
                ref filename,
                line,
                rust_err,
            } => format!("could not read source file \"{}\" at line {} because of error `{:?}`",
                filename, line, rust_err),
            UnexpectedToken {
                ref found,
                ref expected,
            } => format!("found token \"{}\", expected one of {:?}",
                found, expected),
            NoBinding {
                ref name,
            } => format!("no binding found for `{}`", name),
            BindingExists {
                ref name,
            } => format!("duplicate binding of `{}`", name),
            ITypeMismatch {
                ref found,
                ref expected,
            } => format!("found type `{}`, expected type `{}`", found, expected),
            MTypeMismatch {
                ref found,
                ref expected,
            } => format!("found metalogical type `{}`, expected metalogical type `{}`", found, expected),
            UnboundImplication => String::from("implication between non-nullary formulae"),
            UnboundTheorem     => String::from("axiom/theorem accepts logical arguments"),
        }
    }

    pub fn to_console<W: Write>(&self, dest: &mut W, source: &SourceInfo) {
        dest.write(format!("At {}:\n{}{} : {}\n",
            self.location_string(), excerpt(source, &self.location), self.err_type(),
            self.message()).as_bytes()).unwrap();
    }

    pub fn to_console_noexcerpt<W: Write>(&self, dest: &mut W) {
        dest.write(format!("At {}:\n{} : {}\n",
            self.location_string(), self.err_type(), self.message()).as_bytes()).unwrap();
    }
}


pub type Result<T> = result::Result<T, Error>;


#[derive(Debug)]
pub struct SourceInfo {
    filename: String,
    lines: Vec<String>,
    len_so_far: Vec<usize>,
}

impl SourceInfo {
    pub fn new(filename: &str, context: &FileLocation) -> Result<SourceInfo> {
        match File::open(filename.clone()) {
            Ok(source_file) => {
                let mut buf = BufReader::new(source_file);
                let mut lines: Vec<String> = Vec::new();
                let mut len_so_far: Vec<usize> = Vec::new();
                let mut line = String::new();
                loop {
                    match buf.read_line(&mut line) {
                        Ok(amt) => {
                            if amt == 0 { break; }
                            let partial_sum = match len_so_far.last() {
                                Some(ref num) => **num,
                                None          => 0usize,
                            };
                            len_so_far.push(amt + partial_sum);
                            lines.push(line.replace("\r", "\u{200B}"));
                            line.clear();
                        },
                        Err(err) => {
                            return Err(Error::new(FileReadFailure {
                                filename: String::from(filename),
                                line: lines.len(),
                                rust_err: err.kind(),
                            }, context));
                        },
                    }
                }
                Ok(SourceInfo {
                    filename: String::from(filename),
                    lines: lines,
                    len_so_far: len_so_far,
                })
            },
            Err(err) => {
                Err(Error::new(FileOpenFailure {
                    filename: String::from(filename),
                    rust_err: err.kind(),
                }, context))
            }
        }
    }

    pub fn to_file_location(&self, index: usize) -> FileLocation {
        match self.len_so_far.binary_search(&index) {
            Ok(lno)  => FileLocation::new(&self.filename, lno, index - self.len_so_far.get(lno).unwrap()),
            Err(lno) => FileLocation::new(&self.filename, lno, index - self.len_so_far.get(lno - 1).unwrap()),
        }
    }

    pub fn get_line(&self, lno: usize) -> String {
        self.lines.get(lno).unwrap().replace("\r", "").replace("\n", "")
    }
}


#[derive(Clone, Debug)]
pub struct FileLocation {
    filename: String,
    line: usize,
    col: usize,
}

impl FileLocation {
    pub fn new(filename: &str, line: usize, col: usize) -> FileLocation {
        FileLocation {
            filename: String::from(filename),
            line: line,
            col: col
        }
    }

    // pub fn get_source(&self) -> &str {
    //     &self.source
    // }

    // pub fn get_line(&self) -> usize {
    //     self.line
    // }

    // pub fn get_col(&self) -> usize {
    //     self.col
    // }

    pub fn full_string(&self) -> String {
        format!("{}:{}:{}", self.filename, self.line, self.col)
    }
}


fn excerpt(source: &SourceInfo, location: &FileLocation) -> String {
    format!("|\n|\t{}\n|\t{}\n",
        source.get_line(location.line), format!("{}^", String::from(" ").repeat(location.col)))
}


// pub trait ErrorLocated {
//     const ERROR_NAME: &'static str;

//     fn location(&self) -> &FileLocation;

//     fn excerpt(&self) -> &str;

//     fn message(&self) -> String;
// }

// impl<E: ErrorLocated> Error for E {
//     const ERROR_NAME: &'static str = <E as ErrorLocated>::ERROR_NAME;

//     fn location_string(&self) -> String {
//         self.location().full_string()
//     }

//     fn message(&self) -> String {
//         <E as ErrorLocated>::message(&self)
//     }

//     fn to_console<W: Write>(&self, dest: &mut W) {
//         dest.write(format!("At {}:\n|\n|\t{}\n|\t{}\n{} : {}\n", self.location_string(),
//             self.excerpt(), format!("{}^", String::from(" ").repeat(self.location().col)),
//             <Self as Error>::ERROR_NAME, self.message()).as_bytes());
//     }
// }