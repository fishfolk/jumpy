use crate::util::prepend_crate;
use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Lit, LitStr, Path, Token, Type};

struct Signature {
    core_crate: Ident,
    event_type: Type,
}

struct Syntax {
    core_crate: LitStr,
    _comma_1: Token![,],
    event_type: Type,
}

impl Parse for Signature {
    fn parse(stream: ParseStream) -> syn::Result<Self> {
        if stream.is_empty() {
            panic!("The init_resources macro requires three arguments!");
        } else {
            let syntax = Syntax {
                core_crate: stream.parse()?,
                _comma_1: stream.parse()?,
                event_type: stream.parse()?,
            };

            Ok(Signature {
                core_crate: Ident::from_value(&Lit::Str(syntax.core_crate)).unwrap(),
                event_type: syntax.event_type,
            })
        }
    }
}

pub(crate) fn setup_events_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let signature = syn::parse_macro_input!(input as Signature);

    let core_crate = signature.core_crate;
    let event_type = signature.event_type;

    custom_events(&core_crate, event_type)
}

pub(crate) fn custom_events(core_crate: &Ident, event_type: Type) -> proc_macro::TokenStream {
    let res = quote! {
        use #core_crate::glutin::event_loop::EventLoop;
        use #core_crate::glutin::event_loop::EventLoopProxy;

        pub type Event = #core_crate::event::Event<#event_type>;

        static mut EVENT_LOOP_PROXY: Option<EventLoopProxy<Event>> = None;

        fn event_loop_proxy() -> &'static mut EventLoopProxy<Event> {
            unsafe { EVENT_LOOP_PROXY.as_mut().unwrap() }
        }

        pub fn new_event_loop() -> EventLoop<Event> {
            let event_loop = EventLoop::<Event>::with_user_event();
            let proxy = event_loop.create_proxy();

            unsafe { EVENT_LOOP_PROXY = Some(proxy) }

            event_loop
        }

        pub fn try_dispatch_event<E: Into<Event>>(event: E) -> #core_crate::result::Result<()> {
            let event = event.into();
            event_loop_proxy().send_event(event)?;

            Ok(())
        }

        pub fn dispatch_event<E: Into<Event>>(event: E) {
            try_dispatch_event(event).unwrap_or_else(|err| panic!("Error when dispatching event: {}", err));
        }
    };

    proc_macro::TokenStream::from(res)
}
