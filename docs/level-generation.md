# Level Generation

## General Thoughts

- Levels should fit within a set width and height. There may be some empty
  space, but levels should generally fill the space.
- To give a sense of continuity, levels should lie on top of each other, so the
  X-Y coordinates of the staircase from level B1 to B2 should be the same as the
  entrance staircase on B2.
  - It's okay to punt on this to start. Not following this rule would give the
    sense that the levels are staggered, spread over a wider footprint, rather
    than stacked in a single column. For a natural cave system, that could
    actually be more realistic.
- Levels should fit nicely onto a standard 16:9 window.
  - Later I may need to slightly adjust the ratio to leave room for UI.
- About 30 x 30 seems right for tile size.

## Related Work

- [Wave Function Collapse][1]
- [Interview on Brogue's level generation][2]

[1]: https://github.com/mxgmn/WaveFunctionCollapse
[2]: https://www.rockpapershotgun.com/how-do-roguelikes-generate-levels

## Algorithm

1. While the percentage of floor tiles is below the target:
   1. Add a room at a random location in the level region.
   2. Push the room in one of eight random directions until it's not adjacent to
      an existing room.
   3. Crop the room to the level region.
   4. If the level is now too small, discard it and try again. Abort if this
      happens too many times in a row.
2. Connect rooms.
   1. Sort room-room edges by distance.
   2. While not all rooms are connected (determined using a disjoint sets data
      structure) or the most recent connection was too close:
      1. Get the next smallest edge.
      2. Connect these rooms:
         1. If they're aligned vertically or horizontally, use a random straight
            hallway between them.
         2. If they're diagonal from each other, use a random L-shaped connector
            between their nearest two corners.

This produces pretty good results already. I may want to add more loopiness
later, and sometimes the hallways (particularly the L-shaped ones) can cut
through rooms. I'd especially like to prevent hallways from slicing across room
walls, which could look pretty silly if I add doors at some point.
