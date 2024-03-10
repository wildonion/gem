


/* 

middleware can hook into an incoming request process, enabling us to modify requests as well 
as halt request processing to return a response early

let app = App::new()
    .wrap(MiddlewareA)
    .wrap(MiddlewareB)
    .wrap(MiddlewareC)
    .service(service);

                  Request
                     ⭣
╭────────────────────┼────╮
│ MiddlewareC        │    │
│ ╭──────────────────┼───╮│
│ │ MiddlewareB      │   ││
│ │ ╭────────────────┼──╮││
│ │ │ MiddlewareA    │  │││
│ │ │ ╭──────────────┼─╮│││
│ │ │ │ ExtractorA   │ ││││
│ │ │ ├┈┈┈┈┈┈┈┈┈┈┈┈┈┈┼┈┤│││
│ │ │ │ ExtractorB   │ ││││
│ │ │ ├┈┈┈┈┈┈┈┈┈┈┈┈┈┈┼┈┤│││
│ │ │ │ service      │ ││││
│ │ │ ╰──────────────┼─╯│││
│ │ ╰────────────────┼──╯││
│ ╰──────────────────┼───╯│
╰────────────────────┼────╯
                     ⭣
                  Response

*/

pub mod passport;