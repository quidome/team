Feature: Preview imported program data
  The coordinator maps and validates a schedule export before importing it.

  Scenario: Previewing a mapped spreadsheet schedule
    Given a CSV, XLS, XLSX, or ODS export contains game columns
    When the coordinator maps the columns and requests a preview
    Then valid games are shown without changing the Program
    And invalid rows are reported with their source row and field

  Scenario: Resolving conflicting schedule parameters
    Given an imported game matches an existing game by teams, date, and time
    When the coordinator chooses existing or imported values per conflicting field
    Then the merged game is stored with the selected values
