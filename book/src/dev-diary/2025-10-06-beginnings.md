# 06.10.2025 - Beginnings

So if you are following my github page, you might have seen that I recently finished[^1] my first proper simulation project called [Rusty Runways](https://github.com/DennisLent/RustyRunways). This got me thinking about other projects that I could make, that enable simulating complex environments for AIs and after thinking for some time it hit me!

During my thesis, I was initially looking to use the game Sid Meier's Civilization, a really popular 4X game. However, since the game did not have a forward model, nor did it have any good adapters for agents, it was just not feasible to train any agent on it, given limited computational resources.

As such, I realized, I could just try to create my own civ clone albeit more limited. This marks the beginning of this dev journey, where I will try to recreate civilization 6, but in a slimmed down version. My key goals are:

- **Getting world generation working**: World generation should not be too difficult, but I want to allow for the creation of at least 3 distinct map types which I also enjoyed playing, and which allow for interesting scenarios: Pangea, islands, continents. 
- **Gameplay loop and progression**: There are a lot of things to consider and to make sure there are no bugs. In total, I just want to try and port all of the main civ 6 elements that I like, to try and improve the performance and create a fun environment.
- **Customization**: When it comes to AI training games, not all tools work for one problem / goal, therefore I want to design this game with the idea of customization. This could entail maps, factions, progressions etc (lets see what else I can think of)
- **3D rendering**: I am not artist, thats for sure, but getting some basic game setup in 3D so that one can also see what is going on is definetly a must. I did not do that for Rusty Runways, but [civrealm](https://github.com/bigai-ai/civrealm) does allow you to see how the agents play, and so I really want to do that as well.
- **Python**: That is something I have done before, but for AI projects like this, it is important to expose Python bindings, as not everyone is familiar with Rust or even wants to work with it.

## Next steps

This is going to be my main side project for the next couple weeks / months (hopefully not years), but I will take it step by step. I think the first place to start at is the map itself. Making sure we can generate a map and perhaps visualize it using bevy. Once that is in place, then we can see about other next steps.

For gameplay information / balancing, I think I will just take a page out of civilizations book, and take the same values they are using. This way I do not need to fine-tune things myself.

[^1]: As far as a coding project is very truly finished

