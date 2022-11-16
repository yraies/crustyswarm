To run use `cargo run --bin <swarmcli,viz,evoviz> -- <params>`.
For help in the evolutionary processes run `cargo run --bin evoviz -- --help`.

# Binaries and Libraries

| name        | description                                                                                                     |
| --------    | --------------------------------------------------------------------------------------------------------------- |
| evoviz      | Evolves a swarm grammar via opposition-based interactive differential evolution (OIDE).                         |
| rayviz      | Visualizes the development of a swarm grammar.                                                                  |
| core        | Defines and implements the vSG model as well as OIDE operators on it.                                           |
| swarmcli    | Provides many tiny tools to work with vSGs, eg. conversion, oide rebounding or generation of operator analyses. |
| r_oide      | Defines and implements OIDE traits.                                                                             |
| derive_diff | Implementation of derive macros for OIDE traits.                                                                |
