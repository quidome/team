Feature: Preview imported program data
  The coordinator maps and validates a CSV export before importing it.

  Scenario: Previewing a mapped CSV schedule
    Given a CSV export contains game columns
    When the coordinator maps the columns and requests a preview
    Then valid games are shown without changing the Program
    And invalid rows are reported with their source row and field
