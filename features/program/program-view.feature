Feature: Program view
  The coordinator sees one chronological program across games and training.

  Scenario: Viewing the shared program
    Given scheduled games and generated training occurrences exist
    When the coordinator opens the Program view
    Then games and training are shown in chronological order
    And a game shows its location and suggested departure time
    And a cancelled game is retained in the program history but not upcoming Home events
