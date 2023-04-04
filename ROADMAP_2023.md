# Roadmap 2023

## TLDR;

### Goals
- Optional depedency injection
- Async diesel/diesel 2.0
- Improved websocket support
- Improved gRPC support

### Stretch goals
- Improved socket.io support
- Easy-to-use askama middleware
- Easy-to-use frontend library (yew, leptos, etc.)

## Motivation

It's hard to believe that I've been working on this project for so long -- five and a half years at this point. Over time, it's gone through many iterations, the API has changed, the underlying algorithms have changed, but the goal has remained the same; make an easy to use, performant framework with isomorphic middleware/request handlers. To those ends, I think thruster has largely accomplished them.

That being said, what makes a framework a framework is the ecosystem, and libraries that it supports. Thruster has been used in many different applications and products over the years, however it lacks a solid base of extensions/solutions for common problems and use cases. The number of times I've rewritten a database integration, or reintegrated a templating engine at this point has made it so that I'm fairly certain I could write them blindfolded -- why do they not exist as an easy to use part of the framework?

At large, the thruster needs better and easier support for the common use cases that most developers face. This roadmap should serve as a near to mid-term list of milestones to achieve easier integration and better support for common developer use cases.

## Goals

### Inclusion of optional dependency injection (Jab)

Thruster has a great testing suite -- it's easy to take an `App` and individually test routes while still being able to step through the code in a debugger. That being said, having external API integrations and dependencies can mean that code requires compilation-specific flags so that unit tests don't make calls to external providers (or other services in your architecture!) With this in mind, long ago, JabDI was born.

The optional inclusion of Jab will alleviate this concern. Developers will be able to dynamically swap out dummy versions of modules for testing without needing compile time flags, all in a safe, strongly typed way.

### Inclusion of optional diesel 2.0 support

Diesel is a fantastic ORM, really. Over the years, I've built a few [ORM-like packages](https://github.com/trezm/usual), but they pale in scope and capabilities in comparison to diesel. With the async diesel library and Diesel 2.0, it's time that thruster made an easy-to-use module or set of modules to include it. The goal here is a method to easily incorporate diesel into new thruster-based projects.

### Improvements to websocket support

Websockets are a fantastic usecase for thruster's isomorphic middleware design. They're also an important piece of realtime pubsub communications in the modern web. There are a few, experimental, crates for thruster that currently use them, such as the socket.io implementation, however there is no easily exposed implementation of just websockets.

### Improvements to GRPC support

GRPC has been around for a long time, however is still growing in popularity. It's fast, typesafe, and has implementation across many different platforms. The current GRPC support is in [thruster-grpc](https://github.com/thruster-rs/thruster-grpc) and needs some love to make it faster and easier to use.

## Stretch Goals

### Improvements to socket.io support

There are at least two production-grade services that are actively using the [socket.io](https://github.com/thruster-rs/thruster-socketio) library, however the library itself doesn't support some of the key features of socket.io, like polling fallbacks. It has been powerful as a proof of concept, but it would be ideal to make a more fully fledged, to-spec implementation.

The code is also not fantastic in there, and should be cleaned up.

### Inclusion of optional askama support

Askama is a great jinja-based templating library. It's fast and the API is great. Thruster should have a builtin, or easy-to-use, module for including askama without needing to rewrite boilerplate. Ideally, this would be an askama specific middleware that could be dropped into any thruster application.

### Inclusion of a frontend dynamic rendering library

In addition to askama, which provides server side rendering, it would be fantastic to support one (or many!) of the frontend libraries like [Yew](https://github.com/yewstack/yew) or [Leptos](https://github.com/leptos-rs/leptos). This is ideal for those looking to achieve a more "express-like" ecosystem, where they can easily use one language across the stack without worrying (too much) about compilation steps, CDNs, and CORS.

## Summary

If you made it to the bottom of this doc, then I'm impressed! I probably would not have read through the whole thing myself -- heck, I probably would have read the bulletpoints at the top and moved on. This outline will provide plenty of work for myself (as well as any other willing contributors!) for the next year. If I missed a feature that you think is needed, feel free to open up an issue. New contributions of any form are welcome, even if it's just fixing a typo.

Have a great year, and build some cool things my friends.
