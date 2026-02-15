mod matches;
mod simple_int_functions;
mod tuples;

type Result<T = (), E = Box<dyn std::error::Error>> = rolang::Result<T, E>;
