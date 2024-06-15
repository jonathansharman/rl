# Magic

The player should collect magic words (as runes, scroll fragments, whatever)
that can be combined syntactically into simple sentences that form spells.

Example: Take 5 health

Guiding principle: Prefer general-purpose words over hyper-specific ones, to
encourage creativity. E.g., "give that fire" is better than "burn that", and
"move that somewhere" is better than "teleport that".

Possible verbs:

- Give NOUN to TARGET
- Take NOUN from TARGET
- Move TARGET to LOCATION
- Make NOUN at LOCATION
- Destroy NOUN at LOCATION
- Transform NOUN into NOUN

Possible nouns:

- Health
  - This and other nouns could have built-in numerical variants, e.g. 3 health.
    If there are additional numerical modifiers, they would multiply.
- Poison
- Fire
- [monster name]
- [item name]
- [tile name]
- Passage - as in "create passage" (destroy wall) or "destroy passage" (create
  wall)

Targets:

- Self
- That/those
  - Could take an optional numerical modifier for multi-targeting
- Everyone

Should targets and nouns be separate parts of speech?

Locations:

- Here
- There
- Somewhere - random tile (limited by range?)
- Everywhere - ALL TILES?

Numerical modifiers could also be interesting, e.g. "give 3 health to self".

## Restrictions

Besides the limitation of needing to find (and identify) the magic words
themselves, there should be limitations on how many spells the player can use.
There are many possibilities here, such as:

1. Fixed number of uses per spell
2. Mana costs + mana regen (necessitates a food clock or similar)
3. Mana as a consumable resource
4. Global or per-spell cooldowns
5. Limited number of spell slots (equipment system, carry limit, etc.)

I could also mix and match several of these approaches.

I'm interested in trying a no-turn-clock design, where the player is free to
explore as much as they want, with the challenge coming from how to use
consumables, which fights to take, etc. In this case, healing (along with mana
regen, if using mana) need to be finite and not time-based.

Undecided whether words should be consumable or reusable once learned. I
initially envisioned consumable, but either might work. If they are reusable,
then finding a new one is potentially a massive power spike.

## Identification

Words could be initiatially unidentified except for their part of speech,
requiring experimentation to understand them.

To make experimentation more costly, spells could be impossible or at least
expensive to deconstruct into their constituent words. Perhaps a consumable can
be used.
