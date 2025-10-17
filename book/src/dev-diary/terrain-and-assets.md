# Terrain and Assets

It might come as a surprise, but I am not very good at 3D arts. Which is pretty bad, when it comes to wanting to create a 3D simulation / game. Luckily, there are a lot of other great people out there that allow you to use their assets (MASSIVE THANK YOU TO [KENNEY](https://kenney.nl/)). With these assets I was able to create a first prototype that could actually look like the final product. Just to test, I started randomly throwing the assets into the world with no regard, but overall I was pretty happy!

![First Working Assets](book/src/images/first-working-assets.png)

## The Pain of Terrain

Now when it comes to terrain, there are 3 main things we need to consider:

1. Where to place water and land
2. What about the depths of water and the height of the land
3. What about the biomes (temperature and rainfall)

I had already played around a bit with procedural generation using perlin noise, so my approach was going to be the same here and works in 3 stages, going through each of above questions step by step:

**DISCLAIMER**: Right now this is only the algorithm for creating continents maps. Other maps that are typical in civ games, like fractcal, lakes, pangea will follow later, but right now I'll focus on the easy to fix parts.

### 1. Chunking the map

Dependant on the size of the map, we can take N sized chunks. These chunks are then subjugated to RNGsuses will, meaning that we roll a number and if that number is higher or lower than a threshold it is either land or water. This is a really easy approach and is even used in minecraft to determine land and water masses and there is a great explanation [here](https://www.alanzucconi.com/2022/06/05/minecraft-world-generation/).

After each iteration, we can then reduce the chunk size / refine down to determine smaller land / water masses within these bigger one. In that way, we can create more diverse shapes and, when using a deterministic number generator, that means we can always reproduce it with a given seed.

### 2. Perlin Noise

In order to determine where to place mountains and hills, the best and easiest idea is to use perlin noise. This will create deterministic white noise and with some manipulation, we can make it so that the noise will create ridges. These ridges will, similar to the water, act as the decision boundary for mountains. A high, average value in the hex, means that it will be come a mountain, else it remains a normal tile. For this application we need to ensure that the noise we generate does not produce too many mountains.

For water it is a bit easier. Since we already create larger bodies of water, we can simply assume that a tile that is totally surrounded by water, will just become deep water. This is simple and also reduces any randomness.

### 3. Temperature, Rainfall and Whittaker

If you have taken IB Geography in the years between 2015 - 2018 I am 100% sure that you have seen this diagram before

![Whittakers Biome Diagram](book/src/images/Whittakers-Biome-Diagram.png)

Whittaker's Diagram describes how biomes on Earth vary based on precipitation and temperature. This provides and elegant solution for us, as we can use this to determine where biomes will go. To that end, there are 2 ways in which I can go about this:

1. We can use the above mentioned Perlin noise again to create random rainfall and temperature noise, and create varying and diverse ecosystems. Based on the rainfall and temperature we then assign a biome and an asset fitting to that tile. This approach is nice, but also requires some paramter fitting to make sure that biomes are not too large, or too small and that the changes are smooth.
2. A different idea would be to take a similar approach as civ6 does. Civ6 uses a climate and rainfall that is similar to how we have it on Earth. The poles are cold and when you move towards the Equator, it becomes warmer. Rainfall can be varied here and there to allow for rainforests / deserts / marshes / forests etc. However, here I do see that we will still need to use Perlin noise to generate rainfall, else the maps would be all too similar.