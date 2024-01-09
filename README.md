# htmx sorta

A demo project where I learn and play with my "Rust web stack":

* Rust
* Nix flakes for building and dev shell
* [redb](https://github.com/cberner/redb) Rust local key-value store database
* [astra](https://github.com/ibraheemdev/astra) Rust web server library
* [maud](https://github.com/lambda-fairy/maud) Rust html templating library
* [htmx](https://htmx.org/) for dynamic html frontend
* [tailwind](https://tailwindcss.com/)

In essence it's a TODO app, and looks like this (click for a video):

[![Video Preview](https://i.imgur.com/Q8RfOzf.png)](https://i.imgur.com/ctz2bcJ.mp4)

Note: server response slowness is simulated to help improve the UX. The imgur "optimized"
the video and now it's slow and laggy, but I have no energy to fight with
it right now. In reality things are working just fine.

Some of the features: persistent ordered list, with drag'n'drop UX, responsive html,
graceful offline handling, rate limiting.

I think it might be source of inspiration and example for people that would
like to use a similar set of tools for their own projects.

## Running

If you're a Nix flake user you can run `nix run github:dpc/htmx-sorta`. Otherwise,
proceed like with any other Rust project.

## About the stack

I like things simple, small and to the point. Also - it's a bit of a research project,
so I'm not afraid to try out things that are not most popular and trendy.

I'd like to avoid using async Rust, because it's immature and PITA
to use, while blocking IO is perfectly sufficient. That's why I use `astra`,
which is very promising blocking IO Rust web framework/library.

I don't want any C-binding based dependencies, and that's why I'm using `redb`,
which is very promising and lean key value store, in a similar category as e.g. RocksDB.

I don't like DSL, especially looking like combination of Python and HTML, so
`maud` is my favourite HTML framework. Again - fast, lean, to the point.

I'm tired of all the SPAs that are slow heavy and barely work. I'm more of a backend
developer than a frontend developer. I think HTMX is the direction that Web
should have taken long time ago, instead of betting everything on Javascript. I'm not afraid of
adding JS to the code, but it shouldn't be for UX niceness and not the tail wagging
the dog.

I'm not much of a designer or a web developer, but I've done some web projects
over last two decades, and I wholeheartedly agree with Tailwind philosophy.
At the time of writing I wasn't aware how to integrate tailwind with maud well,
so I used `twind` as a client-side Tailwind JIT instead.
[But I was told that maud + tailwind Just Works](https://github.com/rustshop/htmx-sorta/discussions/11),
and switched to this approach.

### Points of Interest

Check [maud templates in fragment.rs](https://github.com/dpc/htmx-sorta/blob/c2d300caafa6a3d72eb7eefcb766b669676ca803/src/fragment.rs),
acting like web-components for both htmx and tailwind.

[Route handlers are in routes.rs](https://github.com/dpc/htmx-sorta/blob/c2d300caafa6a3d72eb7eefcb766b669676ca803/src/routes.rs).
They are a bit low level and manual but hopefully `astra` will get some more convenient approach (e.g. like Axum extractors) in the
near future. [The developer is working on it](https://github.com/ibraheemdev/astra/issues/8). Go say hello if you're interested.

The Nix dev shell I'm using sets up automatically [some cool git hooks](https://github.com/dpc/htmx-sorta/tree/c2d300caafa6a3d72eb7eefcb766b669676ca803/misc/git-hooks). I keep using it in many projects. It includes `semgrep`, `typos` (checking typos), `convco` (conventional commits), linters, and some Rust specific checks.

[`mold` is used to speed up linking, with symbol compression enabled](https://github.com/dpc/htmx-sorta/blob/bc77241ca98e06c7a65e768467d34cbf8bfa8b50/.cargo/config.toml) to make the binary smaller (`6.3 MiB` ATM).

I've "invented" [a variable-length sorting key approach, that simplifies
item sorting.](https://github.com/dpc/htmx-sorta/blob/c2d300caafa6a3d72eb7eefcb766b669676ca803/src/sortid.rs)
It makes generating a key between two other keys a simple and infaliable operation.

I had an idea for [compact and fast, but imprecise pre-rate-limiter](https://github.com/dpc/htmx-sorta/blob/c2d300caafa6a3d72eb7eefcb766b669676ca803/src/rate_limit/pre.rs) that I think would be very fast. It uses a multi-hash approach with buckets of atomic counters, kind of like bloom/cuckoo filters,
to effective take one atomic (relaxed) write on the happy path, and gracefully degrade under heavy load. Did not have time to benchmark it, so 🤷.
Maybe it's stupid.

The rest is somewhat typical. As of the time of writing this there are some known missing things. It's just an demo/research,
don't expect too much.

### License

MIT. Feel free to copy, paste and use, no attribution expected.
