Feature: Review participation and duty history
  The coordinator can review current-season player summaries.

  Scenario: Viewing player participation percentages
    Given attendance records and completed duties exist for the active roster
    When the coordinator opens History
    Then game and training percentages are shown per player
    And completed duty counts are shown per player
    And recorded participation entries are listed

  Scenario: Inspecting audit history
    Given a coordinator import has completed
    When the coordinator opens History
    Then the import action, source, and result counts are visible in the audit list

  Scenario: Inspecting task changes
    Given a manual reminder is created and completed
    When the coordinator opens History
    Then the task creation and status change are visible in the audit list

  Scenario: Inspecting game lifecycle changes
    Given a game is scheduled, rescheduled, and cancelled
    When the coordinator opens History
    Then the game lifecycle actions are visible in the audit list
