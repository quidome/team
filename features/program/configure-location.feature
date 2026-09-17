Feature: Reusable locations
  The coordinator keeps reusable locations for games and training.

  Scenario: Configuring a training location
    Given no location named Home court is stored
    When the coordinator configures Home court with 20 minutes of travel time
    Then the location is stored for reuse
