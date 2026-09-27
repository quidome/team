Feature: Club duty allocation
  The club meeting decides which team covers which jury and referee places at club home games.
  The coordinator records that outcome in the app. Duties are always allocated to a team, never directly to a player.

  Scenario: Suggesting duty places for a club home game
    Given a club home game of Gemengd U12 1
    And the duty rule for U12 games is 2 jury places and 0 referee places
    When the coordinator opens the game's duty allocation
    Then 2 jury places and 0 referee places are suggested

  Scenario: Changing the number of duty places
    Given a club home game with 2 suggested jury places
    When the coordinator sets 3 jury places and 1 referee place
    Then the game has 3 jury places and 1 referee place

  Scenario: Recording the meeting outcome
    Given a club home game on 2026-10-03 with 2 jury places and 1 referee place
    When the coordinator allocates 2 jury places to U18-1 and 1 referee place to U16-2
    Then U18-1 has 2 jury places at that game
    And U16-2 has 1 referee place at that game

  Scenario: Showing places without a team
    Given a club home game has 3 duty places and 2 are allocated to teams
    When the coordinator views the club duty overview
    Then the game shows 1 place without a team

  Scenario: A team sees the duties allocated to it
    Given U16-1 has 1 jury place at the Gemengd U12 1 home game on 2026-10-03
    When the coordinator opens the U16-1 program
    Then the game appears as a duty for U16-1
    And club home games without places for U16-1 do not appear in the U16-1 program
