# Flappy
Rust project for designing a simple game

## Notes

State 
: This is data describing maps. progress, stats and everything else you need
to keep __between__ frames. 

Game loop runs by calling applications 'tick' function every frame.

Traits
: define shared functionality for objects. Similar to interfaces in other languages, which
are used to define a contract.