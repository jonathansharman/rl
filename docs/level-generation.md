# Level Generation

General thoughts:

- Levels should fit within a set width and height. There may be some empty
  space, but levels should generally fill the space.
- To give a sense of continuity, levels should lie on top of each other, so the
  X-Y coordinates of the staircase from level B1 to B2 should be the same as the
  entrance staircase on B2.
- Levels should fit nicely onto a standard 16:9 window.
  - Later I may need to slightly adjust the ratio to leave room for UI.
- About 30 x 30 seems right for tile size.

[This article][1] has a lot of good detail - and Brogue has good level
generation.

[1]: https://www.rockpapershotgun.com/how-do-roguelikes-generate-levels

Algorithm:

1. Pick the stairs spot / spawn point.
2. Generate a structure at that spot.
3. Generate N additional structures, using a relaxation algorithm to push them
   apart.
4. Cull or clip any structures that don't fit within the target area.
5. Connect structures. (How?)
   1. Connect every structure to every other structure where it has line of
      sight.
   2. If there are structures with no line of sight to another structure...?
6. Choose exit point. Should be sufficiently far from the spawn point.
