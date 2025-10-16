# 16.10.2025 - Hexes and Maps

So like any person would do, the first thing I had to do to understand hex grids, was read up about them. Luckily for me, there are a lot of great resources out there, which explain how hex grids work. If you are reading this and are interested, I would suggest you check out this [page](https://www.redblobgames.com/grids/hexagons/implementation.html).

From reading, I wanted to give it a shot at making just the axial version with flat-top orientation, as this most reminded me of civ. However... working on it was more of an issue than I thought. Initially, just making the struct, the few methods to calculate distances, sizes etc was simple, but the biggest hurddle was getting it into bevy and that is where the struggle began. 

To place it into bevy, we need to switch between the axial and world coordinates, which turned out to be just more difficult than I thought. Overall, I got them placed, but never properly aligned, meaning there were small spaces and pockets that were not filled. Apart from just being wrong, it also looked unaesthetic and very unsatisfying.

I tried to tinker around more, but when I got sick I kind of just left it. It was not worth the effort to go and try to reinvent the wheel, when there are already crates that work very well for hexagonal coordinates. After some research, I decided to use [hexx](https://crates.io/crates/hexx). It seems easy enough to use, and is even based on the same resource that I used initially. 

In the end, I managed to get it working with a bit of helps from the docs and trusty old ChatGPT to figure out the camera placements (I really dislike how the camera is handled in bevy). For now, it simply creates a grid of hex tiles based on the size selected. I used the civ6 map sizes as a guideline here:

| Map Size | Dimensions | Players |
|----------|------------|---------|
| Duel     | 44x26      | 2–4     |
| Tiny     | 60x38      | 4/6     |
| Small    | 74x46      | 6/10    |
| Standard | 84x54      | 8/14    |
| Large    | 96x60      | 10/16   |
| Huge     | 106x66     | 12/20   |

## Next steps

Now in order to make this game like civ, we will need to add terrain. For the basic civ setup, there are only a few main tiles that we need. Also, the civ map, as far as I am concerned does not directly follow the whittaker model like here on earth. It might be interesting to try and expand a bit on the type of biomes, using rainfall and temperature to make it more diverse and to smoothen the transitions. 
