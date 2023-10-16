use quote::ToTokens;
use syn::visit::{self, Visit};
use syn::{ItemFn, ExprCall, ExprMethodCall};
use walkdir::WalkDir;
//use std::ffi::OsStr;


#[derive(Clone)]
pub struct FnDef {
    pub file: String,
    pub parent: Option<String>, // name of struct or enum
    pub name: String,
    pub calls: Vec<String>,
}

impl FnDef {
    fn new(file: String, parent: Option<String>, name: String) -> Self {
        Self {
            file,
            parent,
            name,
            calls: vec![],
        }
    }
    
    pub fn to_string(&self, include: &Vec<String>) -> String {
        let mut s = format!("{}\n", self.name);
        for call in &self.calls {
            if !include.contains(call) {
                continue;
            }
            s.push_str("    ");
            s.push_str(call);
            s.push_str("\n");
        }
        s
    }
    
}

pub struct CallsVisitor {
    functions: Vec<FnDef>,
    current_file: String,
    current_parent: Option<String>,
    current_function: Option<FnDef>,
}

impl CallsVisitor {
    pub fn new() -> Self {
        Self {
            functions: vec![],
            current_file: String::new(),
            current_parent: None,
            current_function: None,
        }
    }

    fn start_new_fndef(&mut self, name: String) {
        let full_name = if let Some(parent) = &self.current_parent {
            format!("{}::{}", parent, name)
        } else {
            name
        };
        self.current_function = Some(FnDef::new(
            self.current_file.clone(),
            self.current_parent.clone(),
            full_name,
        ));
    }

    fn push_current_function(&mut self) {
        self.functions.push(self.current_function.take().unwrap());
    }
}

impl<'ast> Visit<'ast> for CallsVisitor {
    fn visit_expr_call(&mut self, expr: &'ast ExprCall) {
        if let Some(the_func) = &mut self.current_function {
            let token_string: String = expr.func.to_token_stream().to_string();
            let tokens: Vec<String> = token_string.split_whitespace().map(String::from).collect();            
            let call = &tokens[tokens.len()-1];
            let call_name: String = tokens.join("");
            // if the name is capitalized then it's probably an enum instance
            // not a function call
            if call.chars().next().unwrap().is_uppercase() {
                return;
            }
            if !the_func.calls.contains(&call_name) {
                the_func.calls.push(call_name);
            }
            visit::visit_expr_call(self, expr);
        }
    }

    fn visit_expr_method_call(&mut self, expr: &'ast ExprMethodCall) {
        if let Some(the_func) = &mut self.current_function {
            let recv = expr.receiver.to_token_stream().to_string();
            let qualif = if recv == "self" {
                if let Some(parent) = &self.current_parent {
                    parent
                } else {
                    panic!("should be a parent. recv: {}", recv);
                }
            } else {
                "?"
            };
            let name = expr.method.to_string();
            let call_name = format!("{}::{}", qualif, name);
            if !the_func.calls.contains(&call_name) {
                the_func.calls.push(call_name);
            }
            visit::visit_expr_method_call(self, expr);
        }
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        let fn_name = item.sig.ident.to_string();
        self.start_new_fndef(fn_name);
        visit::visit_item_fn(self, item);
        self.push_current_function();
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let fn_name = item.sig.ident.to_string();
        self.start_new_fndef(fn_name);
        visit::visit_impl_item_fn(self, item);
        self.push_current_function();
    }
    
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        self.current_parent = Some(item.self_ty.to_token_stream().to_string());
        visit::visit_item_impl(self, item);
        self.current_parent = None;
    }

}


pub fn process_funcs_in_dir(dir_path: &str) -> Vec<FnDef> {
    let mut cv = CallsVisitor::new();
    // walk the directory for each rs file
    for entry in WalkDir::new(dir_path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension() == Some(std::ffi::OsStr::new("rs")) {
            // for the given rs file, make the syntax tree
            let file_path = entry.path();
            let content = std::fs::read_to_string(file_path).unwrap();
            if let Ok(syntax_tree) = syn::parse_str(&content) {
                // find callers in the syntax tree
                cv.current_file = file_path.file_name().unwrap().to_str().unwrap().to_string();
                cv.visit_file(&syntax_tree);
            }
        }
    }
    cv.functions
}
