Feature: Season configuration
  The coordinator establishes the current season before managing team data.

  Scenario: Configuring the 2026–2027 season
    Given no 2026–2027 season is stored
    When the coordinator configures a season starting in 2026
    Then the season is stored with starting year 2026 and ending year 2027
