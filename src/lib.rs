extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(ShinyHandler)]
pub fn stream_handler_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_handler_macro(&ast)
}

fn impl_handler_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl actix::prelude::StreamHandler<Result<actix_web_actors::ws::Message, actix_web_actors::ws::ProtocolError>> for #name {
            fn handle(&mut self, msg: Result<actix_web_actors::ws::Message, actix_web_actors::ws::ProtocolError>, session: &mut Self::Context) {
                // process websocket messages
                match msg {
                    Ok(actix_web_actors::ws::Message::Ping(msg)) => {
                        self.hb = std::time::Instant::now();
                        session.pong(&msg);
                    }
                    Ok(actix_web_actors::ws::Message::Pong(_)) => {
                        self.hb = std::time::Instant::now();
                    }
                    Ok(actix_web_actors::ws::Message::Text(text)) => {
                        let msg: ShinyMsg = serde_json::from_str(&text).expect("Invalid websocket message");
                            match msg.method.as_str() {
                                "init" => {
                                    let initial_inputs = msg.data.keys();
                                    for key in initial_inputs {
                                        self.input.insert(key, msg.data[key].clone());
                                    }
                                    (self.initialize)(self, session)
                                },
                                "update" => {
                                    let changed_inputs = msg.data.keys();
                                    for key in changed_inputs {
                                        if self.check_change(&msg, &key) {
                                            self.event = key.to_string();
                                            (self.update)(self, session)
                                        }
                                    }
                                },
                                _ => {}
                            }
                    },
                    Ok(actix_web_actors::ws::Message::Binary(bin)) => session.binary(bin),
                    Ok(actix_web_actors::ws::Message::Close(reason)) => {
                        session.close(reason);
                        session.stop();
                    }
                    _ => session.stop(),
                }
                (self.tick)(self, session)
            }
        }
    };
    gen.into()
}
