# Motivation

WebAssembly is great, and it's not just for browsers. It's a well
designed, security-conscious virtual machine that has the potential 
to be used anywhere a platform-independent, language-agnostic, sandboxed computing 
environment is needed.

WebAssembly provides a way for a device vendor to become a platform vendor
by providing an API and host environment for end users and third parties.

Web browsers are one example of this, but there are countless other examples:
applications such as Photoshop plugins, text editors, even industrial devices
and robots.

There are other examples where a platform approach makes sense even if the API
isn't intended to be open to end users. For instance, game engines often use scripting 
languages so that their content developers don't need to use the same language as the 
engine developers and to reduce porting costs. Similarly, embedded developers with a 
family of products may want to keep their higher level application code separate from 
their underlying device-specific code, and maybe even use different programming 
languages and teams.

Finally, many vendors face the challenge of securely distributing firmware updates.
Code signing is the first line of defense, but sandboxed firmware can prevent
jailbreaks even if that fails, even on devices without memory protection.

**bobbin-wasm** is a WebAssembly engine designed to enable these applications. It's aim 
is to be small, fast, and secure.