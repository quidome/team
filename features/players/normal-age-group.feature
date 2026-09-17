Feature: Normal age-group calculation
  Team memberships show the normal age group calculated from the season start year.

  Scenario: Two birth years map to U16 for the 2026–2027 season
    Given the season starts in 2026
    When a player was born in 2012 or 2011
    Then the normal age group is U16

  Scenario: The next two birth years map to U18 for the 2026–2027 season
    Given the season starts in 2026
    When a player was born in 2010 or 2009
    Then the normal age group is U18
