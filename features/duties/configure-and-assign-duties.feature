Feature: Assign and confirm team duties
  A team gets duty places in two ways: the club allocates jury and referee places at other games,
  and the team decides how many drivers its own away games need.
  The team agrees who does each place, the coordinator records it, and after the game confirms or corrects who did it.

  Scenario: Configuring driving places for an away game
    Given a scheduled U16-1 away game at 14:30
    When the coordinator sets 2 driving places
    Then the game has 2 driving places for U16-1
    And the suggested departure time remains visible

  Scenario: Suggesting driving places from attendance
    Given 9 players are recorded as present for a U16-1 away game
    When the coordinator opens the game's duties
    Then a number of driving places is suggested from the attendance

  Scenario: Assigning the agreed player to a place
    Given U16-1 has 1 jury place at a club home game
    When the coordinator assigns Avery to the jury place
    Then Avery is assigned to that place

  Scenario: Confirming who did the duty
    Given Avery is assigned to a jury place at a game that has been played
    When the coordinator confirms the assignment
    Then Avery receives completion credit for the duty

  Scenario: Correcting who did the duty
    Given Avery is assigned to a jury place at a game that has been played
    When the coordinator records that Blake did the duty instead
    Then Blake receives completion credit for the duty
    And Avery does not receive completion credit
    And the change remains in the duty history
