Feature: Stable game fixtures and occurrences
  A fixture remains stable when scheduled game occurrences are cancelled or rescheduled.

  Scenario: Scheduling an own home game
    Given U16-1 and U18-1 are configured teams
    And Home court is a reusable location
    When the coordinator creates a fixture with U16-1 at home against U18-1
    And schedules it for 2026-09-05 at 14:30
    Then the fixture and its scheduled occurrence are stored separately
    And the occurrence keeps the 30-minute arrival buffer

  Scenario: Rescheduling a game occurrence
    Given a scheduled occurrence exists for a stable fixture
    When the coordinator reschedules it to 2026-09-12 at 15:00
    Then the original occurrence remains stored as cancelled
    And a new scheduled occurrence is stored for the fixture
    And the suggested departure time is calculated from travel and arrival buffer
