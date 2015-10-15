// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(rustc_private, plugin_registrar, quote, convert)]
#![crate_name = "pcp_lang"]

extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;

mod rust;
mod anonymous_macro;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_syntax_extension(
    rust::token::intern("pcp"),
    rust::SyntaxExtension::NormalTT(Box::new(expand), None, true));
}

fn expand<'cx>(cx: &'cx mut rust::ExtCtxt, _sp: rust::Span,
  tts: &[rust::TokenTree]) -> Box<rust::MacResult + 'cx>
{
  parse(cx, tts.iter().cloned().collect())
}

fn abort_if_errors(cx: &rust::ExtCtxt) {
  cx.parse_sess.span_diagnostic.handler.abort_if_errors();
}

fn parse<'cx>(cx: &'cx rust::ExtCtxt,
  tts: Vec<rust::TokenTree>) -> Box<rust::MacResult + 'cx>
{
  let expander = anonymous_macro::Expander::new(cx, tts);
  expander.expand()
}
