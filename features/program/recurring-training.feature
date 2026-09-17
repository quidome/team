Feature: Recurring training occurrences
  The coordinator can define a recurring training series for the current season.

  Scenario: Generating Tuesday training occurrences through the series end date
    Given a training series starts on Tuesday 2026-08-18
    And it ends on Tuesday 2026-09-01
    When the coordinator generates its occurrences
    Then occurrences exist on 2026-08-18, 2026-08-25, and 2026-09-01
    And each occurrence keeps the configured time, duration, and location
