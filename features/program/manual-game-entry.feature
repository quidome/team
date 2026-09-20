Feature: Manually enter a game
  The coordinator can add a game without importing a schedule file.

  Scenario: Adding a scheduled away game
    Given the Away court location is configured
    When the coordinator enters U16-1 away against U18-1 at 14:30 on 2026-08-15
    Then the game appears in the Program
    And its suggested departure time uses the travel time and arrival buffer

  Scenario: Handling a missing game occurrence
    Given a game occurrence no longer exists
    When the coordinator tries to cancel or reschedule it
    Then the API reports that the game occurrence was not found
