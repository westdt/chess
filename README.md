# Chess
A very simple chess game with a chess bot to play against.
The purpose of this project is threefold: learning, demonstrating my skills, and just having fun. As of right now, it's really just a project I made in my free time in 1 week. There are a lot of changes I would have made if this was intended to be an application for other people's use. Since it is just a fun project there are liberties I have taken here that I would not normally take.
#### Learning
- Learn Rust better: Rust is a language that I have been interested ever since I learned about it. Previously I had learned Java before moving onto C/C++ since I was interested in video game development. I really fell in love with C because it was so stripped down compared to the bloat of Java, however C has many many problems that have since been solved by other languages. I view Rust as the successor to C in many ways and I think that Rust has a very bright future ahead of it.

- Learn TypeScript and how to use it side-by-side with Rust: Learning JavaScript is something that almost every developer tries at some point. Most frontends use JavaScript and learning how to use it enables me to work on and understand thousands of frontends. Learning how to integrate this with Rust was a challenge that I approached in a very non-typical way at first, and then learned the proper way later.
 
- ✨Tauri! ✨   Tauri is an amazing toolkit for implementing a frontend with a Rust backend as well as having many more features that I have yet to apply. In my opinion, Tauri is a real gamechanger and I see Rust + Tauri + React as being an extremely powerful and promising option for the future.
 
- React: React is obviously a highly in demand skillset that is used by the vast majority of JS-based frontend applications. This chess app has taught me how to use state (as well as the limitations of state).

#### Demonstrating my skills
- General development: This project shows my knowledge of basic coding concepts like variables, types, structs, logic gates/control flow, and many more.

- Rust : This project demonstrates my ability to code idiomatic Rust using key Rust features such as borrowing, match statements, and immutability, as well as just a general understanding of Rust's syntax.

- Frontend: This project demonstrates my ability to write a frontend in React, implementing events, state, etc.

#### Having fun!
- I do enjoy coding and learning all of these languages and how to implement them to create a functional chess game has been fun for me.

- I have always wanted to write a chess bot. Now that I have written a frontend I can write one!

## Current Features
- Chess move generation and validation
    - Normal moves
    - En passant
    - Castling
    - Check validation
    - Checkmate detection (broken currently)
- Fully playable chess bot
    - Uses minimax algorithm with alpha-beta pruning
- React frontend
- Rust backend

## Planned Features
- ✅ Move to React: Plain TS/HTML/CSS was easier to whip something up quickly but I plan on moving it to React.
- Algebraic notation: Need to add a feature to detect moves and determine the appropriate notation.
- Better chess bot: The current chess bot is very poor 😅 I plan to make it better and add book openings
- ✅ Better bot support: I would like to be able to have different bots that can be easily selected and used.
- Restructure/optimization: Need to restructure and create a better utility class that engines can be based on. Also need to have a translation layer between the backend and frontend (for example, "pid" is needed by React to keep track of piece elements, but it is completely useless as far as the actual chess code goes, meaning it is bloat on the engine.)
- ✅ Animations: Switching to React I had to temporarily sacrifice the animations I had in the previous version. I have an idea for how I can fix it but I just have not done it yet.
- ✅ Async backend: AI needs to run asynchronously. Right now it is causing major problems by running synchronously.

# Dev notes
- The main engine is not optimized at all. I would like to optimize it however the top priority for the main engine is accuracy/legibility, not speed. Realistically the entire purpose of this engine is just to have a 100% accurate system of verifying what moves are legitimate or not. AI engines _can_ be built on top of this, however it would be better if they had a faster backend and ONLY used the main engine for final verification. Maybe if I develop a better engine, I can eventually port that to the main engine, but it is not a priority now as long as the main engine functions accurately, since if it only needs to be run once for every turn, performance can be sacrificed.
