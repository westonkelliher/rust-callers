use quote::ToTokens;
use syn::visit::{self, Visit};
use syn::{ItemFn, ExprCall, ExprMethodCall};
use walkdir::WalkDir;
//use std::ffi::OsStr;


#[derive(Clone)]
pub struct FnDef {
    file: String,
    parent: Option<String>, // name of struct or enum
    pub name: String,
    calls: Vec<String>,
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
        let locale = if let Some(parent) = &self.parent {
            format!("{}::{}", self.file, parent)
        } else {
            self.file.clone()
        };
        let mut s = format!("{}<{}>\n", self.name, locale);
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
        self.current_function = Some(FnDef::new(
            self.current_file.clone(),
            self.current_parent.clone(),
            name,
        ));
    }

    fn push_current_function(&mut self) {
        self.functions.push(self.current_function.take().unwrap());
    }
}

impl<'ast> Visit<'ast> for CallsVisitor {
    fn visit_expr_call(&mut self, expr: &'ast ExprCall) {
        if let Some(the_func) = &mut self.current_function {
            let call_name = expr.func.to_token_stream().to_string() // TODO: is this sufficient?
                .split_whitespace().last().unwrap().to_owned();
            if !the_func.calls.contains(&call_name) {
                the_func.calls.push(call_name);
            }
            visit::visit_expr_call(self, expr);
        }
    }

    fn visit_expr_method_call(&mut self, expr: &'ast ExprMethodCall) {
        if let Some(the_func) = &mut self.current_function {
            let call_name = expr.method.to_string()
                .split_whitespace().last().unwrap().to_owned();
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
            let syntax_tree: syn::File = syn::parse_str(&content).unwrap();
            // find callers in the syntax tree
            cv.current_file = file_path.file_name().unwrap().to_str().unwrap().to_string();
            cv.visit_file(&syntax_tree);
        }
    }
    cv.functions
}
