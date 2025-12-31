use crate::types::Symbol;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

impl Scope {
    pub fn new(parent: Option<usize>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
            children: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct SymbolTable {
    pub scopes: Vec<Scope>,
    pub current_scope_id: usize,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        let root = Scope::new(None);
        Self {
            scopes: vec![root],
            current_scope_id: 0,
        }
    }

    pub fn create_scope(&mut self) -> usize {
        let new_id = self.scopes.len();
        let new_scope = Scope::new(Some(self.current_scope_id));
        self.scopes.push(new_scope);

        // Add as child to current scope
        self.scopes[self.current_scope_id].children.push(new_id);

        new_id
    }

    pub fn enter_scope(&mut self, id: usize) {
        if id < self.scopes.len() {
            self.current_scope_id = id;
        } else {
            panic!("Scope ID {} out of bounds", id);
        }
    }

    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope_id].parent {
            self.current_scope_id = parent;
        } else {
            panic!("Cannot exit global scope");
        }
    }

    pub fn define(&mut self, symbol: Symbol) -> Result<(), String> {
        let scope = &mut self.scopes[self.current_scope_id];
        if scope.symbols.contains_key(&symbol.name) {
            return Err(format!(
                "Symbol '{}' already defined in current scope",
                symbol.name
            ));
        }
        scope.symbols.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        let mut current = self.current_scope_id;
        loop {
            let scope = &self.scopes[current];
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }
            if let Some(parent) = scope.parent {
                current = parent;
            } else {
                break;
            }
        }
        None
    }

    pub fn current_level(&self) -> usize {
        let mut level = 0;
        let mut current = self.current_scope_id;
        while let Some(parent) = self.scopes[current].parent {
            level += 1;
            current = parent;
        }
        level
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph SymbolTable {\n");
        dot.push_str("    node [shape=record, fontname=\"Arial\", fontsize=10];\n");
        dot.push_str("    rankdir=TB;\n");

        for (id, scope) in self.scopes.iter().enumerate() {
            let label_header = if let Some(parent) = scope.parent {
                format!("Scope {} (Parent: {})", id, parent)
            } else {
                format!("Scope {} (Global)", id)
            };

            let mut rows = Vec::new();
            rows.push(format!("<B>{}</B>", label_header));

            // Sort symbols by name for consistent output
            let mut symbols: Vec<_> = scope.symbols.values().collect();
            symbols.sort_by(|a, b| a.name.cmp(&b.name));

            for sym in symbols {
                let desc = match &sym.kind {
                    crate::types::SymbolType::Constant { val } => {
                        format!("const {} = {}", sym.name, val)
                    }
                    crate::types::SymbolType::Variable { level, addr } => {
                        format!("var {} (L:{}, A:{})", sym.name, level, addr)
                    }
                    crate::types::SymbolType::Procedure { level, addr } => {
                        format!("proc {} (L:{}, A:{})", sym.name, level, addr)
                    }
                };
                rows.push(desc);
            }

            // Escape special characters in label
            let label = rows.join(" | ").replace(">", "&gt;").replace("<", "&lt;").replace("{", "\\{").replace("}", "\\}");
            // Re-enable bold tags (hacky but simple)
            let label = label.replace("&lt;B&gt;", "<B>").replace("&lt;/B&gt;", "</B>");
            
            dot.push_str(&format!("    scope_{} [label=\"{{ {} }}\"];\n", id, label));

            for child_id in &scope.children {
                dot.push_str(&format!("    scope_{} -> scope_{};\n", id, child_id));
            }
        }

        dot.push_str("}\n");
        dot
    }
}
