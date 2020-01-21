use proc_macro2 as pm2;
use quote::{ToTokens};
use std::cell::RefCell;
use crate::shared::{Parameters};


pub struct ControlGen<'a> {
    ty: syn::Ident,
    member: &'a syn::Ident,
    params: RefCell<Parameters>,  
}

impl<'a> ToTokens for ControlGen<'a> {

    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let control_params = self.params.borrow();
        let member = self.member;
        let ty = &self.ty;
    
        let ids: Vec<&syn::Ident> = control_params.params.iter().map(|p| &p.ident).collect();
        let values: Vec<&syn::Expr> = control_params.params.iter().map(|p| &p.e).collect();
    
        let control_tk = quote! {
            nwg::#ty::builder()
                #(.#ids(#values))*
                .build(&mut data.#member)?;
        };

        control_tk.to_tokens(tokens);
    }

}

pub fn control_parameters(field: &syn::Field) -> (Vec<syn::Ident>, Vec<syn::Expr>) {
    let member = match field.ident.as_ref() {
        Some(m) => m,
        None => unreachable!()
    };

    let nwg_control = |attr: &&syn::Attribute| {
        attr.path.get_ident()
          .map(|id| id == "nwg_control" )
          .unwrap_or(false)
    };

    let attr = match field.attrs.iter().find(nwg_control) {
        Some(attr) => attr,
        None => unreachable!()
    };

    let ctrl: Parameters = match syn::parse2(attr.tokens.clone()) {
        Ok(a) => a,
        Err(e) => panic!("Failed to parse field #{}: {}", member, e)
    };

    let params = ctrl.params;
    let mut names = Vec::with_capacity(params.len());
    let mut exprs = Vec::with_capacity(params.len());

    for p in params {
        if p.ident == "ty" {
            continue;
        }

        names.push(p.ident);
        exprs.push(p.e);
    }

    (names, exprs)
}

pub fn expand_flags(member_name: &syn::Ident, ty: &syn::Ident, flags: syn::Expr) -> syn::Expr {
    let flags_type = format!("{}Flags", ty);
    
    let flags_value = match &flags {
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Str(value) => value,
            other => panic!("Compressed flags must str, got {:?} for control {}", other, member_name)
        },
        other => panic!("Compressed flags must str, got {:?} for control {}", other, member_name)
    };

    let flags = flags_value.value();
    let splitted: Vec<&str> = flags.split('|').collect();

    let flags_count = splitted.len() - 1;
    let mut final_flags: String = String::with_capacity(100);
    for (i, value) in splitted.into_iter().enumerate() {
        final_flags.push_str("nwg::");
        final_flags.push_str(&flags_type);
        final_flags.push_str("::");
        final_flags.push_str(value);

        if i != flags_count {
            final_flags.push('|');
        }
    }

    match syn::parse_str(&final_flags) {
        Ok(e) => e,
        Err(e) => panic!("Failed to parse flags value for control {}: {}", member_name, e)
    }
}
