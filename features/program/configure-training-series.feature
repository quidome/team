Feature: Training series persistence
  The coordinator stores a recurring training series and its generated occurrences.

  Scenario: Storing a Tuesday training series
    Given Home court is a reusable location
    When the coordinator stores a Tuesday series from 2026-08-18 through 2026-09-01
    Then the series and three generated occurrences are stored
