Feature: Season-specific team membership
  The coordinator records how a player belongs to a team during a season.

  Scenario: Recording an active primary U16-1 membership
    Given Avery is a player with association ID 12345
    And U16-1 is a configured team
    And the 2026–2027 season is configured
    When the coordinator records Avery as an active primary player who trains and plays
    Then the membership is stored for U16-1 in the 2026–2027 season
