# Motivation

Short version: WebAssembly is great, but it's not just for browsers. It's a well
designed, security-conscious virtual machine specification that has the potential 
to be used anywhere a platform-independent, language-independent, sandboxed computing 
environment is needed.

In some cases this is because a platform vendor wants to provide an API and hosting
environment to end users or third parties while keeping the underlying platform secure.
Web browsers are the prime example of this, but there are countless other examples:
applications such as Photoshop offering plugins, text editors, even industrial devices
such as PLCs and robots that exist so that they can be programmed to do things.

In others, these may be internal platforms that face some of the same challenges. For
instance, game engines often use scripting languages so that their content developers
don't need to use the same language as the engine developers and to reduce porting costs. 
Embedded device developers often need to distribute firmware updates but run the risk of having third party binaries jailbreak their devices.

WebAssembly is the first VM that has come along in a long time (maybe ever) that isn't 
controlled by a single entity and that has the chance of gaining widespread usage because of the coalition of web browser developers that are supporting its development. It's also been carefully designed with security and ease of implementation in mind, and is fairly minimalist in philosophy.

**bobbin-wasm** is an attempt to build a WebAssembly engine that proves that it's suitable for these non-browser applications, particularly in resource-constrained embedded applications. It's aim is to be small, fast, and secure.